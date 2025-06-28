import { execSync } from 'node:child_process'
import fsp from 'node:fs/promises'
import path from 'node:path'

import AdmZip from 'adm-zip'
import fs from 'fs-extra'
import fetch from 'node-fetch'
import { extract } from 'tar'

import { downloadFile, getFetchOptions, getOpenlistArchMap, getRcloneArchMap, platformIdentifierMap } from './utils.js'

const cwd = process.cwd()
const target = process.argv.find(arg => arg.startsWith('--target='))?.slice(9)
const platformArch = target ? platformIdentifierMap[target] : `${process.platform}-${process.arch}`
const [platform, arch] = platformArch.split('-')
const isWin = platform === 'win32'

console.log(`platform: ${platform}, arch: ${arch}`)

const sidecarHost =
  target ||
  execSync('rustc -vV')
    .toString()
    .match(/(?<=host: ).+(?=\s*)/g)[0]

if (!getOpenlistArchMap[platformArch]) {
  throw new Error(`Unsupported target: ${platformArch}. Supported: ${Object.keys(getOpenlistArchMap).join(', ')}`)
}

// Rclone version management
let rcloneVersion = 'v1.70.1'
const rcloneVersionUrl = 'https://github.com/rclone/rclone/releases/latest/download/version.txt'

async function getLatestRcloneVersion() {
  try {
    const response = await fetch(rcloneVersionUrl, getFetchOptions())
    rcloneVersion = (await response.text()).trim().replace('rclone ', '') || '1.70.1'
    console.log(`Latest rclone version: ${rcloneVersion}`)
  } catch (error) {
    console.log('Error fetching latest rclone version:', error.message)
  }
}

// openlist version management
let openlistVersion = 'v4.0.3'

async function getLatestOpenlistVersion() {
  try {
    const response = await fetch(
      'https://api.github.com/repos/OpenListTeam/OpenList/releases/latest',
      getFetchOptions()
    )
    const data = await response.json()
    openlistVersion = data.tag_name || 'v4.0.3'
    console.log(`Latest OpenList version: ${openlistVersion}`)
  } catch (error) {
    console.log('Error fetching latest OpenList version:', error.message)
  }
}

const createBinaryInfo = (name, archMap, baseUrl, version = '') => {
  const zipName = archMap[platformArch]
  return {
    name,
    targetFile: `${name}-${sidecarHost}${isWin ? '.exe' : ''}`,
    exeFile: `${name}${isWin ? '.exe' : ''}`,
    zipFile: zipName,
    downloadURL: `${baseUrl}${version ? `/${version}` : ''}/${zipName}`
  }
}

// OpenList service URL
const serviceUrl = `https://github.com/OpenListTeam/openlist-desktop-service/releases/download/${sidecarHost}`

const getServiceInfo = exeName => {
  const ext = isWin ? '.exe' : ''
  const suffix = '-' + sidecarHost
  return {
    file: exeName + suffix + ext,
    downloadURL: `${serviceUrl}/${exeName}${ext}`
  }
}

// SimpleSC.dll
const resolvePlugin = async () => {
  const url = 'https://nsis.sourceforge.io/mediawiki/images/e/ef/NSIS_Simple_Service_Plugin_Unicode_1.30.zip'
  const TEMP_DIR = path.join(cwd, 'temp')
  const tempDir = path.join(TEMP_DIR, 'SimpleSC')
  const tempZip = path.join(tempDir, 'NSIS_Simple_Service_Plugin_Unicode_1.30.zip')
  const tempDll = path.join(tempDir, 'SimpleSC.dll')
  const pluginDir = path.join(process.env.APPDATA, 'Local/NSIS')
  const pluginPath = path.join(pluginDir, 'SimpleSC.dll')
  await fs.mkdir(pluginDir, { recursive: true })
  await fs.mkdir(tempDir, { recursive: true })
  if (fs.existsSync(pluginPath)) return
  try {
    if (!fs.existsSync(tempZip)) {
      await downloadFile(url, tempZip)
    }
    const zip = new AdmZip(tempZip)

    zip.extractAllTo(tempDir, true)
    await fsp.cp(tempDll, pluginPath, { recursive: true, force: true })
    console.log(`SimpleSC.dll copied to ${pluginPath}`)
  } finally {
    await fsp.rm(tempDir, { recursive: true, force: true })
  }
}

async function resolveSidecar(binInfo) {
  const { name, targetFile, zipFile, exeFile, downloadURL } = binInfo
  const binaryDir = path.join(cwd, 'src-tauri', 'binary')
  const binaryPath = path.join(binaryDir, targetFile)

  await fs.mkdir(binaryDir, { recursive: true })
  const zipPath = path.join(binaryDir, zipFile)

  try {
    await downloadFile(downloadURL, zipPath)

    if (zipFile.endsWith('.zip')) {
      const zip = new AdmZip(zipPath)
      zip.extractAllTo(binaryDir, true)

      if (name === 'rclone') {
        const extractedDir = path.join(binaryDir, zipFile.replace('.zip', ''))
        await fs.rename(path.join(extractedDir, exeFile), binaryPath)
        await fs.remove(extractedDir)
      } else {
        await fs.rename(path.join(binaryDir, exeFile), binaryPath)
      }
    } else {
      await extract({ cwd: binaryDir, file: zipPath })
      await fs.rename(path.join(binaryDir, exeFile), binaryPath)
    }
    await fs.remove(zipPath)
    await fs.chmod(binaryPath, 0o755)
  } catch (err) {
    console.error(`Error preparing "${name}":`, err.message)
    await fs.rm(binaryPath, { recursive: true, force: true })
    throw err
  }
}

async function resolveService(resourceInfo, isChmod = true, defaultMode = 0o755) {
  const { file, downloadURL } = resourceInfo
  const resourceDir = path.join(cwd, 'src-tauri', 'binary')
  const resourcePath = path.join(resourceDir, file)

  await fs.mkdir(resourceDir, { recursive: true })

  try {
    await downloadFile(downloadURL, resourcePath)
    if (isChmod) {
      await fs.chmod(resourcePath, defaultMode)
    }
    console.log(`"${file}" downloaded to ${resourcePath}`)
  } catch (err) {
    console.error(`Error preparing "${file}":`, err.message)
    await fs.rm(resourcePath, { recursive: true, force: true })
    throw err
  }
}

async function retryTask(name, fn, maxRetries = 5) {
  for (let i = 0; i < maxRetries; i++) {
    try {
      console.log(`task::${name} try ${i}`)
      await fn()
      return
    } catch (err) {
      console.log(`task::${name} try ${i} ==`, err.message)
      if (i === maxRetries - 1) throw err
    }
  }
}

async function main() {
  await getLatestOpenlistVersion()
  await retryTask('openlist', () =>
    resolveSidecar(
      createBinaryInfo(
        'openlist',
        getOpenlistArchMap,
        'https://github.com/OpenListTeam/OpenList/releases/download',
        openlistVersion
      )
    )
  )

  await retryTask('rclone', async () => {
    await getLatestRcloneVersion()
    await resolveSidecar(
      createBinaryInfo(
        'rclone',
        getRcloneArchMap(rcloneVersion),
        'https://github.com/rclone/rclone/releases/download',
        rcloneVersion
      )
    )
  })
  if (isWin) {
    await resolvePlugin()
  }
  await resolveService(getServiceInfo('install-openlist-service'))
  await resolveService(getServiceInfo('openlist-desktop-service'))
  await resolveService(getServiceInfo('uninstall-openlist-service'))
}

main().catch(console.log)
