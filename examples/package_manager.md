to get user packages

## dnf

```bash
dnf repoquery --installed --queryformat '%{name} %{version} %{reason}\n' | grep "User"
```

output sample:

```text
aajohan-comfortaa-fonts 3.105 User
akmod-nvidia 595.80 User
anaconda-install-env-deps 44.30 User
anaconda-live 44.30 User
asusctl 6.3.8 User
asusctl-rog-gui 6.3.8 User
brightnessctl 0.5.1 User
clang 22.1.8 User
clang-devel 22.1.8 User
clash-verge 2.5.1 User
cmake 4.3.0 User
code 1.128.0 User
cuda-toolkit 13.3.1 User
dnf5 5.4.2.1 User
dracut-live 108 User
expat-devel 2.8.1 User
fastfetch 2.65.2 User
fcitx5 5.1.19 User
fcitx5-chinese-addons 5.1.12 User
fcitx5-configtool 5.1.13 User
fcitx5-gtk 5.1.6 User
fcitx5-qt 5.1.13 User
fcitx5-rime 5.1.13 User
fedora-release-kde-desktop 44 User
filesystem 3.18 User
flameshot 13.3.0 User
fuse 2.9.9 User
gcc 16.1.1 User
gh 2.94.0 User
ghostty 1.3.1 User
golang 1.26.4 User
grub2-efi-ia32-modules 2.12 User
grub2-efi-x64-modules 2.12 User
grubby 8.40 User
gtk3-devel 3.24.52 User
htop 3.4.1 User
igt-gpu-tools 2.4 User
intel-media-driver 26.1.5 User
isomd5sum 1.2.5 User
java-latest-openjdk 26.0.1.0.8 User
kde-l10n 17.08.3 User
kernel 7.0.13 User
kernel 7.0.14 User
kernel 7.1.3 User
kernel-core 7.0.13 User
kernel-core 7.0.14 User
kernel-core 7.1.3 User
kernel-modules 7.0.13 User
kernel-modules 7.0.14 User
kernel-modules 7.1.3 User
kernel-modules-core 7.0.13 User
kernel-modules-core 7.0.14 User
kernel-modules-core 7.1.3 User
kernel-modules-extra 7.0.13 User
kernel-modules-extra 7.0.14 User
kernel-modules-extra 7.1.3 User
kitty 0.47.1 User
kmod-nvidia-7.0.13-200.fc44.x86_64 595.80 User
kmod-nvidia-7.0.14-201.fc44.x86_64 595.80 User
kmod-nvidia-7.1.3-200.fc44.x86_64 595.80 User
kwayland-integration 6.7.2 User
libavcodec-freeworld 8.1.2 User
libreoffice-draw 26.2.4.2 User
libreoffice-math 26.2.4.2 User
libva-utils 2.23.0 User
libxkbcommon-devel 1.13.1 User
livesys-scripts 0.9.6 User
lua 5.4.8 User
mediawriter 5.3.1 User
memtest86+ 8.10 User
mihomo-party 1.9.6 User
moby-engine 29.6.0 User
nodejs24-npm 11.8.0 User
obs-studio 32.1.1 User
pcre2-devel 10.47 User
plasma-welcome-fedora 6.3.4 User
postgresql-contrib 18.3 User
postgresql-server 18.3 User
python3-pip 26.0.1 User
rpmfusion-free-release 44 User
rpmfusion-nonfree-release 44 User
snapd 2.76 User
sqlite 3.51.2 User
systemd-devel 259.7 User
systemd-oomd-defaults 259.7 User
terra-release 44 User
udisks2-btrfs 2.11.1 User
vim-enhanced 9.2.780 User
vlc 3.0.23 User
xorg-x11-drv-nvidia-cuda 595.80 User
```

## apt

```bash
apt-mark showmanual |
  xargs -r dpkg-query -W -f='${binary:Package} ${Version}\n'
```

output sample:

```text
bash 5.3-2ubuntu1
btrfs-progs 6.17.1-1build1
cracklib-runtime 2.9.6-5.2build3
dash 0.5.12-12ubuntu3
diffutils 1:3.12-1
dmeventd 2:1.02.205-2ubuntu3
efibootmgr 18-4ubuntu1
fastfetch 2.57.1+dfsg-1ubuntu1
findutils 4.10.0-3build2
grep 3.12-1
grub-efi-amd64-bin 2.14-2ubuntu1
grub-efi-amd64-signed 1.215+2.14-2ubuntu1
grub-efi-amd64-unsigned 2.14-2ubuntu1
grub-gfxpayload-lists 0.7build3
grub-pc 2.14-2ubuntu2
grub-pc-bin 2.14-2ubuntu2
gzip 1.14-1~exp2ubuntu1.1
hostname 3.25build1
hyphen-en-ca 0.10ubuntu3
hyphen-fi 0.10ubuntu3
hyphen-ga 0.10ubuntu3
hyphen-id 1:25.2.3-1build1
init 1.69
jfsutils 1.1.15-7
keyutils 1.6.3-6ubuntu3
kubuntu-desktop 1.496
kubuntu-wallpapers 26.04.3
language-pack-en 1:26.04+20260417
language-pack-en-base 1:26.04+20260417
libaio1t64:amd64 0.3.113-8build1
libboost-python1.90.0 1.90.0-6ubuntu1
libcalamares3.3 3.3.14-0ubuntu25
libcrack2:amd64 2.9.6-5.2build3
libdevmapper-event1.02.1:amd64 2:1.02.205-2ubuntu3
liblvm2cmd2.03:amd64 2.03.31-2ubuntu3
libopenblas0:amd64 0.3.32+ds-5
libpwquality-common 1.4.5-5build1
libpwquality1:amd64 1.4.5-5build1
libyaml-cpp0.8:amd64 0.8.0+dfsg-9
linux-generic 7.0.0-27.27
lvm2 2.03.31-2ubuntu3
ncurses-base 6.6+20251231-1
ncurses-bin 6.6+20251231-1
openoffice.org-hyphenation 0.10ubuntu3
qemu-guest-agent 1:10.2.1+ds-1ubuntu3.1
shim-signed 1.59+15.8-0ubuntu2
spice-vdagent 0.23.0-1
thin-provisioning-tools 1.1.0-4ubuntu2
ubuntu-minimal 1.570
ubuntu-standard 1.570
wamerican 2020.12.07-4build1
wbritish 2020.12.07-4build1
xfsprogs 6.18.0-3
```
