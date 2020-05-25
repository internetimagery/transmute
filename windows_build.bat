REM usage: windows_build python3
cargo build --release --features %1
mv target\release\transmute.dll target\release\transmute.pyd
