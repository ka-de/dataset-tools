# Change directory to dataset-tools
Set-Location -Path "C:\Users\kade\code\dataset-tools"

# Execute the cargo build command
cargo build --workspace -Z build-std --target x86_64-pc-windows-msvc --release

# Define source and destination directories
$sourceDir = "C:\Users\kade\code\dataset-tools\target\x86_64-pc-windows-msvc\release"
$destDir = "C:\Users\kade\Desktop\apps\dataset-tools"

# Copy only *.exe files from the root of source to destination
Get-ChildItem -Path $sourceDir -File -Filter "*.exe" | Copy-Item -Destination $destDir -Force

# Run the compress-exe.exe
#& "C:\Users\kade\code\dataset-tools\target\x86_64-pc-windows-msvc\release\compress-exe.exe" "C:\Users\kade\Desktop\apps\dataset-tools\"

