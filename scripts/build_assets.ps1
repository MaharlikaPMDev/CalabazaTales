param([string]$Root = (Split-Path -Parent $PSScriptRoot))

Add-Type -AssemblyName System.Drawing

function New-Canvas([int]$Width, [int]$Height) {
    return [System.Drawing.Bitmap]::new($Width, $Height, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
}

function Save-Png($Bitmap, [string]$Path) {
    $directory = Split-Path -Parent $Path
    New-Item -ItemType Directory -Force -Path $directory | Out-Null
    $Bitmap.Save($Path, [System.Drawing.Imaging.ImageFormat]::Png)
    $Bitmap.Dispose()
}

function Color([string]$Hex) { return [System.Drawing.ColorTranslator]::FromHtml($Hex) }

function Draw-PixelArt([string[]]$Rows, [hashtable]$Palette, [string]$Path) {
    $bitmap = New-Canvas $Rows[0].Length $Rows.Count
    for ($y = 0; $y -lt $Rows.Count; $y++) {
        for ($x = 0; $x -lt $Rows[$y].Length; $x++) {
            $key = [string]$Rows[$y][$x]
            if ($Palette.ContainsKey($key)) { $bitmap.SetPixel($x, $y, (Color $Palette[$key])) }
        }
    }
    Save-Png $bitmap $Path
}

$heartFrame = @(
    '..GG.GG..', '.GSSGSSG.', 'GSSSSSSSG', 'GSSSSSSSG', '.GSSSSSG.', '..GSSSG..', '...GSG...', '....G....', '.........'
)
$heartFull = @(
    '..GG.GG..', '.GRRGRRG.', 'GRRRRRRRG', 'GRRWWRRRG', '.GRRRRRG.', '..GRRRG..', '...GRG...', '....G....', '.........'
)
$heartHalf = @(
    '..GG.GG..', '.GRRGSSG.', 'GRRRSSSSG', 'GRRWSSSSG', '.GRRSSSG.', '..GRSSG..', '...GSG...', '....G....', '.........'
)
$shieldFrame = @(
    '.GGGGGGG.', 'GSSSSSSSG', 'GSSSSSSSG', 'GSSSSSSSG', '.GSSSSSG.', '.GGSSSGG.', '..GSSSG..', '...GSG...', '....G....'
)
$shieldFull = @(
    '.GGGGGGG.', 'GBBBBBBBG', 'GBBWWBBBG', 'GBBBBBBBG', '.GBBBBBG.', '.GGBBBGG.', '..GBBBG..', '...GBG...', '....G....'
)
$shieldHalf = @(
    '.GGGGGGG.', 'GBBBSSSSG', 'GBBWSSSSG', 'GBBBSSSSG', '.GBBSSSG.', '.GGBSSGG.', '..GBSSG..', '...GSG...', '....G....'
)
$palette = @{ G = '#E9B94E'; S = '#29303A'; R = '#D92F46'; W = '#FFD4BE'; B = '#2879D6' }
$goldHeartPalette = @{ G = '#E9B94E'; S = '#29303A'; R = '#F4C85C'; W = '#FFF1A8'; B = '#2879D6' }
$frozenHeartPalette = @{ G = '#D7ECFF'; S = '#29303A'; R = '#4FA7E8'; W = '#EAF8FF'; B = '#2879D6' }
$poisonHeartPalette = @{ G = '#E9B94E'; S = '#29303A'; R = '#5A9C45'; W = '#CFEFA8'; B = '#2879D6' }
$witherHeartPalette = @{ G = '#E9B94E'; S = '#17191D'; R = '#555A64'; W = '#AEB4C0'; B = '#2879D6' }

$java = Join-Path $Root 'resource-pack-java/assets/minecraft/textures/gui/sprites/hud'
$bedrock = Join-Path $Root 'resource-pack-bedrock/textures/ui'
Draw-PixelArt $heartFrame $palette (Join-Path $java 'heart/container.png')
Draw-PixelArt $heartFull $palette (Join-Path $java 'heart/full.png')
Draw-PixelArt $heartHalf $palette (Join-Path $java 'heart/half.png')
Draw-PixelArt $heartFrame $palette (Join-Path $java 'heart/container_blinking.png')
Draw-PixelArt $heartFrame $palette (Join-Path $java 'heart/container_hardcore.png')
Draw-PixelArt $heartFrame $palette (Join-Path $java 'heart/container_hardcore_blinking.png')
Draw-PixelArt $heartFull $palette (Join-Path $java 'heart/full_blinking.png')
Draw-PixelArt $heartHalf $palette (Join-Path $java 'heart/half_blinking.png')
Draw-PixelArt $heartFull $palette (Join-Path $java 'heart/full_hardcore.png')
Draw-PixelArt $heartHalf $palette (Join-Path $java 'heart/half_hardcore.png')
Draw-PixelArt $heartFull $palette (Join-Path $java 'heart/full_hardcore_blinking.png')
Draw-PixelArt $heartHalf $palette (Join-Path $java 'heart/half_hardcore_blinking.png')
Draw-PixelArt $heartFull $goldHeartPalette (Join-Path $java 'heart/absorbing_full.png')
Draw-PixelArt $heartHalf $goldHeartPalette (Join-Path $java 'heart/absorbing_half.png')
Draw-PixelArt $heartFull $frozenHeartPalette (Join-Path $java 'heart/frozen_full.png')
Draw-PixelArt $heartHalf $frozenHeartPalette (Join-Path $java 'heart/frozen_half.png')
Draw-PixelArt $heartFull $poisonHeartPalette (Join-Path $java 'heart/poisoned_full.png')
Draw-PixelArt $heartHalf $poisonHeartPalette (Join-Path $java 'heart/poisoned_half.png')
Draw-PixelArt $heartFull $witherHeartPalette (Join-Path $java 'heart/withered_full.png')
Draw-PixelArt $heartHalf $witherHeartPalette (Join-Path $java 'heart/withered_half.png')
Draw-PixelArt $shieldFrame $palette (Join-Path $java 'armor_empty.png')
Draw-PixelArt $shieldFull $palette (Join-Path $java 'armor_full.png')
Draw-PixelArt $shieldHalf $palette (Join-Path $java 'armor_half.png')

Draw-PixelArt $heartFrame $palette (Join-Path $bedrock 'heart_background.png')
Draw-PixelArt $heartFull $palette (Join-Path $bedrock 'heart.png')
Draw-PixelArt $heartFull $palette (Join-Path $bedrock 'heart_new.png')
Draw-PixelArt $heartFull $palette (Join-Path $bedrock 'heart_blink.png')
Draw-PixelArt $heartFull $palette (Join-Path $bedrock 'heart_flash.png')
Draw-PixelArt $heartHalf $palette (Join-Path $bedrock 'heart_half.png')
Draw-PixelArt $heartHalf $palette (Join-Path $bedrock 'heart_flash_half.png')
Draw-PixelArt $heartFull $goldHeartPalette (Join-Path $bedrock 'absorption_heart.png')
Draw-PixelArt $heartHalf $goldHeartPalette (Join-Path $bedrock 'absorption_heart_half.png')
Draw-PixelArt $heartFull $frozenHeartPalette (Join-Path $bedrock 'freeze_heart.png')
Draw-PixelArt $heartFull $frozenHeartPalette (Join-Path $bedrock 'freeze_heart_flash.png')
Draw-PixelArt $heartHalf $frozenHeartPalette (Join-Path $bedrock 'freeze_heart_half.png')
Draw-PixelArt $heartHalf $frozenHeartPalette (Join-Path $bedrock 'freeze_heart_flash_half.png')
Draw-PixelArt $shieldFrame $palette (Join-Path $bedrock 'armor_empty.png')
Draw-PixelArt $shieldFull $palette (Join-Path $bedrock 'armor_full.png')
Draw-PixelArt $shieldHalf $palette (Join-Path $bedrock 'armor_half.png')

function Draw-Bar([string]$Path, [bool]$Filled, [int]$Width, [int]$Height) {
    $bitmap = New-Canvas $Width $Height
    $gold = Color '#E9B94E'; $silver = Color '#B8C4D4'; $dark = Color '#151A22'; $red = Color '#D92F46'; $light = Color '#FF6878'
    for ($x = 0; $x -lt $Width; $x++) {
        $bitmap.SetPixel($x, 0, $gold); $bitmap.SetPixel($x, $Height - 1, $gold)
        for ($y = 1; $y -lt $Height - 1; $y++) { $bitmap.SetPixel($x, $y, $(if ($Filled) { $red } else { $dark })) }
    }
    for ($y = 0; $y -lt $Height; $y++) { $bitmap.SetPixel(0, $y, $silver); $bitmap.SetPixel($Width - 1, $y, $silver) }
    if ($Filled -and $Height -gt 3) { for ($x = 2; $x -lt $Width - 2; $x++) { $bitmap.SetPixel($x, 1, $light) } }
    Save-Png $bitmap $Path
}

$boss = Join-Path $Root 'resource-pack-java/assets/minecraft/textures/gui/sprites/boss_bar'
Draw-Bar (Join-Path $boss 'red_background.png') $false 182 5
Draw-Bar (Join-Path $boss 'red_progress.png') $true 182 5
Draw-Bar (Join-Path $boss 'notched_10_background.png') $false 182 5
Draw-Bar (Join-Path $boss 'notched_10_progress.png') $true 182 5
Draw-Bar (Join-Path $bedrock 'empty_progress_bar.png') $false 182 8
Draw-Bar (Join-Path $bedrock 'filled_progress_bar.png') $true 182 8

function Draw-PackIcon([string]$Path) {
    $bitmap = New-Canvas 128 128
    $g = [System.Drawing.Graphics]::FromImage($bitmap)
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $g.Clear((Color '#121722'))
    $goldPen = [System.Drawing.Pen]::new((Color '#E9B94E'), 7)
    $blueBrush = [System.Drawing.SolidBrush]::new((Color '#245EAA'))
    $redBrush = [System.Drawing.SolidBrush]::new((Color '#D92F46'))
    $goldBrush = [System.Drawing.SolidBrush]::new((Color '#F4C85C'))
    $shield = [System.Drawing.Point[]]@((New-Object System.Drawing.Point 28,20),(New-Object System.Drawing.Point 100,20),(New-Object System.Drawing.Point 94,83),(New-Object System.Drawing.Point 64,111),(New-Object System.Drawing.Point 34,83))
    $g.FillPolygon($blueBrush, $shield); $g.DrawPolygon($goldPen, $shield)
    $heart = [System.Drawing.Point[]]@((New-Object System.Drawing.Point 64,78),(New-Object System.Drawing.Point 43,57),(New-Object System.Drawing.Point 45,42),(New-Object System.Drawing.Point 55,36),(New-Object System.Drawing.Point 64,44),(New-Object System.Drawing.Point 73,36),(New-Object System.Drawing.Point 83,42),(New-Object System.Drawing.Point 85,57))
    $g.FillPolygon($redBrush, $heart)
    $font = [System.Drawing.Font]::new('Georgia', 18, [System.Drawing.FontStyle]::Bold)
    $g.DrawString('Ds', $font, $goldBrush, 48, 78)
    $g.Dispose(); $goldPen.Dispose(); $blueBrush.Dispose(); $redBrush.Dispose(); $goldBrush.Dispose(); $font.Dispose()
    Save-Png $bitmap $Path
}

Draw-PackIcon (Join-Path $Root 'resource-pack-java/pack.png')
Draw-PackIcon (Join-Path $Root 'resource-pack-bedrock/pack_icon.png')
