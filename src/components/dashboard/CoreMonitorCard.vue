<template>
  <Card variant="elevated" hover class="service-monitor-card">
    <template #header>
      <h3>{{ t('dashboard.coreMonitor.title') }}</h3>
      <div class="status-indicator" :class="{ online: isCoreRunning, offline: !isCoreRunning }">
        <div class="pulse-dot"></div>
        <span>{{ isCoreRunning ? t('dashboard.coreMonitor.online') : t('dashboard.coreMonitor.offline') }}</span>
      </div>
    </template>
    <div class="heartbeat-section">
      <div class="heartbeat-header">
        <h4></h4>
        <div class="metrics" v-if="isCoreRunning">
          <span class="metric info">
            <Globe :size="14" />
            Port: {{ openlistCoreStatus.port || 5244 }}
          </span>
          <span class="metric info">
            <Activity :size="14" />
            {{ t('dashboard.coreMonitor.responseTime') }}: {{ responseTime }}ms
          </span>
          <span
            class="metric"
            :class="{
              healthy: avgResponseTime < 100,
              warning: avgResponseTime >= 100 && avgResponseTime < 500,
              error: avgResponseTime >= 500
            }"
          >
            {{ avgResponseTime }}ms avg
          </span>
          <span class="metric success">{{ successRate }}% uptime</span>
        </div>
      </div>

      <div class="heartbeat-chart" ref="chartContainer">
        <svg :width="chartWidth" :height="chartHeight" class="heartbeat-svg">
          <defs>
            <pattern id="grid" width="20" height="20" patternUnits="userSpaceOnUse">
              <path d="M 20 0 L 0 0 0 20" fill="none" :stroke="gridColor" stroke-width="0.5" opacity="0.3" />
            </pattern>
          </defs>
          <rect width="100%" height="100%" fill="url(#grid)" />

          <path :d="heartbeatPath" fill="none" :stroke="lineColor" stroke-width="2" class="heartbeat-line" />

          <circle
            v-for="(point, index) in visibleDataPoints"
            :key="index"
            :cx="point.x"
            :cy="point.y"
            :r="point.isHealthy ? 3 : 4"
            :fill="point.isHealthy ? lineColor : '#ef4444'"
            class="data-point"
            @mouseover="showTooltip(point, $event)"
            @mouseleave="hideTooltip"
          />
        </svg>

        <div v-if="tooltip.show" class="tooltip" :style="{ left: tooltip.x + 'px', top: tooltip.y + 'px' }">
          <div class="tooltip-content">
            <div class="tooltip-time">{{ tooltip.time }}</div>
            <div class="tooltip-value">{{ tooltip.value }}ms</div>
            <div class="tooltip-status" :class="tooltip.status">{{ tooltip.statusText }}</div>
          </div>
        </div>
      </div>
    </div>
  </Card>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick, watch } from 'vue'
import { useAppStore } from '../../stores/app'
import { useTranslation } from '../../composables/useI18n'
import Card from '../ui/Card.vue'
import { Globe, Activity } from 'lucide-vue-next'

const { t } = useTranslation()
const store = useAppStore()

const chartContainer = ref<HTMLElement>()
const chartWidth = ref(400)
const chartHeight = ref(120)
const dataPoints = ref<Array<{ timestamp: number; responseTime: number; isHealthy: boolean }>>([])
const responseTime = ref(0)
const startTime = ref(Date.now())
const monitoringInterval = ref<number>()

const tooltip = ref({
  show: false,
  x: 0,
  y: 0,
  time: '',
  value: '',
  status: '',
  statusText: ''
})

const isCoreRunning = computed(() => store.isCoreRunning)
const openlistCoreStatus = computed(() => store.openlistCoreStatus)

const avgResponseTime = computed(() => {
  if (dataPoints.value.length === 0) return 0
  const sum = dataPoints.value.reduce((acc, point) => acc + point.responseTime, 0)
  return Math.round(sum / dataPoints.value.length)
})

const successRate = computed(() => {
  if (dataPoints.value.length === 0) return 100
  const healthyPoints = dataPoints.value.filter(point => point.isHealthy).length
  return Math.round((healthyPoints / dataPoints.value.length) * 100)
})

const visibleDataPoints = computed(() => {
  const maxPoints = Math.floor(chartWidth.value / 10)
  const recent = dataPoints.value.slice(-maxPoints)

  return recent.map((point, index) => {
    const x = (index / (recent.length - 1 || 1)) * (chartWidth.value - 20) + 10
    const normalizedResponse = Math.min(point.responseTime / 1000, 1) // Normalize to 0-1 (0-1000ms)
    const y = chartHeight.value - 20 - normalizedResponse * (chartHeight.value - 40)

    return {
      x,
      y,
      isHealthy: point.isHealthy,
      responseTime: point.responseTime,
      timestamp: point.timestamp
    }
  })
})

const heartbeatPath = computed(() => {
  if (visibleDataPoints.value.length < 2) return ''

  let path = `M ${visibleDataPoints.value[0].x} ${visibleDataPoints.value[0].y}`

  for (let i = 1; i < visibleDataPoints.value.length; i++) {
    const prev = visibleDataPoints.value[i - 1]
    const curr = visibleDataPoints.value[i]
    const cpx1 = prev.x + (curr.x - prev.x) * 0.5
    const cpx2 = curr.x - (curr.x - prev.x) * 0.5
    path += ` C ${cpx1} ${prev.y} ${cpx2} ${curr.y} ${curr.x} ${curr.y}`
  }

  return path
})

const lineColor = computed(() => {
  if (avgResponseTime.value < 100) return '#10b981'
  if (avgResponseTime.value < 500) return '#f59e0b'
  return '#ef4444'
})

const gridColor = computed(() => {
  return document.documentElement.classList.contains('dark') ? '#374151' : '#e5e7eb'
})

const checkServiceHealth = async () => {
  await store.refreshServiceStatus()
  if (!isCoreRunning.value) {
    dataPoints.value.push({
      timestamp: Date.now(),
      responseTime: 0,
      isHealthy: false
    })
    responseTime.value = 0
    return
  }

  const startTime = Date.now()

  try {
    await store.refreshServiceStatus()

    const endTime = Date.now()
    const responseTimeMs = endTime - startTime
    const isHealthy = responseTimeMs < 1000

    dataPoints.value.push({
      timestamp: endTime,
      responseTime: responseTimeMs,
      isHealthy
    })

    responseTime.value = responseTimeMs

    if (dataPoints.value.length > 100) {
      dataPoints.value = dataPoints.value.slice(-100)
    }
  } catch (error) {
    dataPoints.value.push({
      timestamp: Date.now(),
      responseTime: 5000,
      isHealthy: false
    })
  }
}

const showTooltip = (point: any, event: MouseEvent) => {
  tooltip.value = {
    show: true,
    x: event.offsetX + 10,
    y: event.offsetY - 10,
    time: new Date(point.timestamp).toLocaleTimeString(),
    value: point.responseTime.toString(),
    status: point.isHealthy ? 'healthy' : 'unhealthy',
    statusText: point.isHealthy ? t('dashboard.coreMonitor.healthy') : t('dashboard.coreMonitor.unhealthy')
  }
}

const hideTooltip = () => {
  tooltip.value.show = false
}

const updateChartSize = () => {
  if (chartContainer.value) {
    chartWidth.value = chartContainer.value.clientWidth
  }
}

onMounted(async () => {
  await nextTick()
  updateChartSize()

  if (isCoreRunning.value) {
    startTime.value = Date.now()
  }

  monitoringInterval.value = window.setInterval(checkServiceHealth, (store.settings.app.monitor_interval || 5) * 1000)
  window.addEventListener('resize', updateChartSize)
})

onUnmounted(() => {
  if (monitoringInterval.value) {
    clearInterval(monitoringInterval.value)
  }
  window.removeEventListener('resize', updateChartSize)
})

watch(isCoreRunning, (newValue: boolean, oldValue: boolean) => {
  if (newValue && !oldValue) {
    startTime.value = Date.now()
  }
})
</script>

<style scoped>
.status-indicator {
  display: flex;
  align-items: center;
  gap: 0.625rem;
  font-size: 0.875rem;
  font-weight: 600;
  padding: 0.5rem 0.875rem;
  border-radius: 20px;
  background: rgba(255, 255, 255, 0.8);
  backdrop-filter: blur(10px);
  border: 1px solid rgba(226, 232, 240, 0.6);
  transition: all 0.2s ease;
}

@media (prefers-color-scheme: dark) {
  .status-indicator {
    background: rgba(30, 41, 59, 0.8);
    border-color: rgba(100, 116, 139, 0.3);
  }
}

.status-indicator.online {
  color: rgb(16, 185, 129);
  background: rgba(16, 185, 129, 0.1);
  border-color: rgba(16, 185, 129, 0.2);
}

.status-indicator.offline {
  color: rgb(239, 68, 68);
  background: rgba(239, 68, 68, 0.1);
  border-color: rgba(239, 68, 68, 0.2);
}

.pulse-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: currentColor;
}

.status-indicator.online .pulse-dot {
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
}

.heartbeat-section {
  margin-bottom: 2rem;
}

.heartbeat-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1.5rem;
  padding-bottom: 0.75rem;
  border-bottom: 2px solid rgba(226, 232, 240, 0.6);
}

@media (prefers-color-scheme: dark) {
  .heartbeat-header {
    border-color: rgba(100, 116, 139, 0.3);
  }
}

.heartbeat-header h4 {
  margin: 0;
  font-size: 1.125rem;
  font-weight: 600;
  color: rgb(51, 65, 85);
  letter-spacing: -0.025em;
}

@media (prefers-color-scheme: dark) {
  .heartbeat-header h4 {
    color: rgb(226, 232, 240);
  }
}

.metrics {
  display: flex;
  gap: 1.25rem;
  font-size: 0.8125rem;
  font-weight: 600;
}

.metric {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.5rem 0.875rem;
  border-radius: 20px;
  font-weight: 600;
  backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.2);
  transition: all 0.2s ease;
}

.metric:hover {
  transform: translateY(-1px);
}

.metric.info {
  background: rgba(100, 116, 139, 0.15);
  color: rgb(71, 85, 105);
  border-color: rgba(100, 116, 139, 0.3);
}

.metric.healthy {
  background: rgba(16, 185, 129, 0.15);
  color: rgb(5, 150, 105);
  border-color: rgba(16, 185, 129, 0.3);
}

.metric.warning {
  background: rgba(251, 191, 36, 0.15);
  color: rgb(217, 119, 6);
  border-color: rgba(251, 191, 36, 0.3);
}

.metric.error {
  background: rgba(239, 68, 68, 0.15);
  color: rgb(220, 38, 38);
  border-color: rgba(239, 68, 68, 0.3);
}

.metric.success {
  background: rgba(59, 130, 246, 0.15);
  color: rgb(37, 99, 235);
  border-color: rgba(59, 130, 246, 0.3);
}

@media (prefers-color-scheme: dark) {
  .metric.info {
    background: rgba(100, 116, 139, 0.2);
    color: rgb(203, 213, 225);
  }

  .metric.healthy {
    background: rgba(16, 185, 129, 0.2);
    color: rgb(110, 231, 183);
  }

  .metric.warning {
    background: rgba(251, 191, 36, 0.2);
    color: rgb(252, 211, 77);
  }

  .metric.error {
    background: rgba(239, 68, 68, 0.2);
    color: rgb(252, 165, 165);
  }

  .metric.success {
    background: rgba(59, 130, 246, 0.2);
    color: rgb(147, 197, 253);
  }
}

.heartbeat-chart {
  position: relative;
  width: 100%;
  height: 140px;
  background: linear-gradient(135deg, rgba(248, 250, 252, 0.8) 0%, rgba(241, 245, 249, 0.9) 100%);
  border-radius: 12px;
  overflow: hidden;
  border: 1px solid rgba(226, 232, 240, 0.6);
  backdrop-filter: blur(10px);
}

@media (prefers-color-scheme: dark) {
  .heartbeat-chart {
    background: linear-gradient(135deg, rgba(15, 23, 42, 0.8) 0%, rgba(30, 41, 59, 0.9) 100%);
    border-color: rgba(100, 116, 139, 0.3);
  }
}

.heartbeat-svg {
  width: 100%;
  height: 100%;
}

.heartbeat-line {
  filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.1));
}

.data-point {
  cursor: pointer;
  transition: r 0.2s ease;
}

.data-point:hover {
  r: 5;
}

.tooltip {
  position: absolute;
  z-index: 10;
  pointer-events: none;
}

.tooltip-content {
  background: #111827;
  color: white;
  padding: 0.5rem;
  border-radius: 6px;
  font-size: 0.75rem;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
}

.tooltip-time {
  font-weight: 500;
  margin-bottom: 0.25rem;
}

.tooltip-value {
  color: #93c5fd;
}

.tooltip-status.healthy {
  color: #6ee7b7;
}

.tooltip-status.unhealthy {
  color: #fca5a5;
}

@media (max-width: 768px) {
  .heartbeat-chart {
    height: 100px;
  }

  .heartbeat-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 0.75rem;
  }

  .metrics {
    flex-wrap: wrap;
    gap: 0.75rem;
  }

  .metric {
    padding: 0.375rem 0.625rem;
    font-size: 0.75rem;
  }
}
</style>
