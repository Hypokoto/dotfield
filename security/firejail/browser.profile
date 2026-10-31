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

# Deny access to sensitive credentials
blacklist ${HOME}/.ssh
blacklist ${HOME}/.gnupg
blacklist ${HOME}/.aws
blacklist ${HOME}/.dotfield

# Whitelist safe paths
whitelist ${HOME}/Downloads
whitelist ${HOME}/.config/fontconfig
whitelist ${HOME}/.fonts
whitelist ${HOME}/.local/share/fonts

# System access controls
nogroups
nosound
no3d
