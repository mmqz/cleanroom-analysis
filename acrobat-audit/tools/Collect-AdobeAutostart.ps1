# Collect-AdobeAutostart.ps1 — Adobe 常驻面只读审计收集器
# 用途: 在 Windows 机器上枚举 Adobe 相关的计划任务/服务/Run键/进程/DNS缓存, 输出 JSON 证据
# 用法: powershell -ExecutionPolicy Bypass -File .\Collect-AdobeAutostart.ps1 [-OutFile .\adobe-audit.json]
# 原则: 只读。不修改任何系统状态(不停服务/不删任务/不改注册表)。
param(
    [string]$OutFile = ".\adobe-autostart-audit.json"
)

$ErrorActionPreference = "SilentlyContinue"
$result = [ordered]@{
    collected_at = (Get-Date).ToString("s")
    host          = $env:COMPUTERNAME
    user          = [System.Security.Principal.WindowsIdentity]::GetCurrent().Name
    scheduled_tasks = @()
    services        = @()
    run_keys        = @()
    processes       = @()
    dns_cache       = @()
    files           = @()
}

# 1) 计划任务 (重点: AdobeGCInvoker-*, Adobe Acrobat Update Task, Adobe Update)
Write-Host "[1/6] Scheduled tasks..." -ForegroundColor Cyan
foreach ($t in Get-ScheduledTask) {
    if ($t.TaskName -match "Adobe|ARM|GCInvoker|Genuine" -or
        $t.TaskPath -match "Adobe") {
        $result.scheduled_tasks += [ordered]@{
            path     = $t.TaskPath + $t.TaskName
            state    = $t.State.ToString()
            triggers = (@($t.Triggers) | ForEach-Object { $_.CimClass.CimClassName }) -join ","
            execute  = ($t.Actions | ForEach-Object { "$($_.Execute) $($_.Arguments)" }) -join " | "
            author   = $t.Author
        }
    }
}

# 2) 服务 (重点: AdobeARMservice, AGS, CC)
Write-Host "[2/6] Services..." -ForegroundColor Cyan
foreach ($s in Get-CimInstance Win32_Service) {
    if ($s.Name -match "Adobe" -or $s.DisplayName -match "Adobe" -or
        $s.PathName -match "Adobe|armsvc") {
        $result.services += [ordered]@{
            name        = $s.Name
            display     = $s.DisplayName
            start_mode  = $s.StartMode
            state       = $s.State
            path        = $s.PathName
            start_name  = $s.StartName   # 验证 LocalSystem 权限
        }
    }
}

# 3) Run 自启动键 (HKLM + HKCU, 32/64 视图)
Write-Host "[3/6] Run keys..." -ForegroundColor Cyan
$runPaths = @(
    "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Run",
    "HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Run",
    "HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Run",
    "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce",
    "HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce"
)
foreach ($p in $runPaths) {
    $k = Get-ItemProperty -Path $p
    if ($k) {
        foreach ($prop in $k.PSObject.Properties) {
            if ($prop.Name -match "Adobe|Acro|ARM" -or
                [string]$prop.Value -match "Adobe|Acrotray|CollabSync|AdobeARM") {
                $result.run_keys += [ordered]@{
                    key   = $p
                    name  = $prop.Name
                    value = [string]$prop.Value
                }
            }
        }
    }
}

# 4) 运行中的 Adobe 进程
Write-Host "[4/6] Processes..." -ForegroundColor Cyan
foreach ($p in Get-Process) {
    if ($p.Name -match "Adobe|Acro|armsvc|arm|CCX|CoreSync|IPCBroker|node") {
        $path = $null
        try { $path = $p.Path } catch {}
        if ($path -match "Adobe" -or $p.Name -match "Adobe|Acro|armsvc") {
            $result.processes += [ordered]@{
                name  = $p.Name
                pid   = $p.Id
                path  = $path
                cpu   = [math]::Round($p.CPU, 1)
                ws_mb = [math]::Round($p.WorkingSet64 / 1MB, 1)
            }
        }
    }
}

# 5) DNS 缓存中的 Adobe 域 (近期实际连接过的端点)
Write-Host "[5/6] DNS cache (Adobe domains)..." -ForegroundColor Cyan
foreach ($e in Get-DnsClientCache) {
    if ($e.Entry -match "adobe|adobegenuine|armmf|demdex|typekit") {
        $result.dns_cache += [ordered]@{
            entry = $e.Entry
            type  = $e.Type
            data  = $e.Data
        }
    }
}

# 6) 关键文件存在性 (AGS / ARM / 崩溃报告器)
Write-Host "[6/6] Key files..." -ForegroundColor Cyan
$paths = @(
    "C:\Program Files (x86)\Common Files\Adobe\AdobeGCClient\AdobeGCClient.exe",
    "C:\Program Files (x86)\Common Files\Adobe\ARM\1.0\AdobeARM.exe",
    "C:\Program Files (x86)\Common Files\Adobe\ARM\armsvc.exe",
    "C:\Program Files\Adobe\Acrobat DC\Acrobat\Acrobat.exe",
    "C:\Program Files\Adobe\Acrobat DC\Acrobat\CRClient.dll",
    "C:\Program Files\Adobe\Acrobat DC\Acrobat\AdobeCollabSync.exe"
)
foreach ($f in $paths) {
    if (Test-Path $f) {
        $fi = Get-Item $f
        $sig = Get-AuthenticodeSignature $f
        $result.files += [ordered]@{
            path          = $f
            version       = $fi.VersionInfo.FileVersion
            last_write    = $fi.LastWriteTime.ToString("s")
            signature     = $sig.Status.ToString()
            signer        = $sig.SignerCertificate.Subject
        }
    }
}

$result | ConvertTo-Json -Depth 5 | Out-File -FilePath $OutFile -Encoding UTF8
Write-Host "`nDone -> $OutFile (只读收集, 未做任何修改)" -ForegroundColor Green
Write-Host "审计要点: AdobeGCInvoker-1.0 任务 / Adobe Acrobat Update Task / "`
          "AdobeARMservice(LocalSystem) / Run: Acrobat Assistant / CRClient.dll 挂接" -ForegroundColor Yellow
