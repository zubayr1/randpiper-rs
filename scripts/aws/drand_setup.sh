# Script that must run on AWS

# Update packages
sudo pacman -Syu --noconfirm

# Install go
sudo pacman -S go git --noconfirm

git clone https://github.com/drand/drand
cd drand
make build