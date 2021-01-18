sudo pacman -Syu --noconfirm

sudo pacman -S git --noconfirm

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > install-rust.sh

bash install-rust.sh -y
source $HOME/.cargo/env

git clone https://github.com/zhtluo/randpiper-rs.git
cd randpiper-rs

make release