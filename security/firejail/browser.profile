# Firejail browser/untrusted app sandbox profile
# Managed by Dotfield

# Restrict Linux capabilities
caps.drop all
nonewprivs
noroot
seccomp

# Filesystem sandboxing
private-tmp
private-dev
disable-mnt
machine-id
tracelog

# Browsers need Unix sockets plus normal IPv4/IPv6 networking, but not raw
# packet, Bluetooth, or other socket families.
protocol unix,inet,inet6,netlink

# Deny access to sensitive host credentials and dotfield metadata
blacklist ${HOME}/.ssh
blacklist ${HOME}/.gnupg
blacklist ${HOME}/.aws
blacklist ${HOME}/.dotfield
blacklist ${HOME}/.pki
blacklist ${HOME}/.local/share/keyrings
blacklist ${HOME}/.config/keepassxc

# Whitelist font caches
mkdir ${HOME}/.config/fontconfig
whitelist ${HOME}/.config/fontconfig

mkdir ${HOME}/.fonts
whitelist ${HOME}/.fonts

mkdir ${HOME}/.local/share/fonts
whitelist ${HOME}/.local/share/fonts

# Dedicated isolated sandbox storage & downloads
mkdir ${HOME}/.local/share/dotfield-sandbox
whitelist ${HOME}/.local/share/dotfield-sandbox
mkdir ${HOME}/.local/share/dotfield-sandbox/browser-profile
whitelist ${HOME}/.local/share/dotfield-sandbox/browser-profile
mkdir ${HOME}/.local/share/dotfield-sandbox/downloads
whitelist ${HOME}/.local/share/dotfield-sandbox/downloads

# System and display access controls (Xephyr X11 server isolation)
# Note: To block access to the host X11 abstract Unix domain socket (@/tmp/.X11-unix/X0),
# Firejail requires a network namespace. Launch via '--net=<interface>' (e.g. --net=wlan0 or --net=eth0),
# or pass '--net=none' if the sandboxed application does not require network access.
nogroups
x11 xephyr

# Compositor IPC isolation (prevent untrusted apps from reaching host Wayland and compositor control sockets)
blacklist ${RUNUSER}/niri*
blacklist ${RUNUSER}/sway*
blacklist ${RUNUSER}/wayland-*

# D-Bus isolation (block access to host Secret Service, Keyring, and session bus)
dbus-user none
dbus-system none
