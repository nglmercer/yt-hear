## 🐧 Linux FAQ & Troubleshooting

### Q: The music won't play, shows "Video format not supported," or keeps skipping to the next track indefinitely. How do I fix this?

**A:** This is caused by missing multimedia codecs on your system.

Unlike Windows or macOS, the Linux version of this app relies on **WebKitGTK** and **GStreamer** to handle audio and video playback. Many Linux distributions do not pre-install proprietary codecs (needed for MP4, AAC, and H.264 content) due to licensing reasons.

To fix this, you simply need to install the "Bad" and "Ugly" GStreamer plugin sets.

#### 🛠️ Solution: Install the required dependencies

Run the command corresponding to your Linux distribution:

**Ubuntu / Debian / Linux Mint / Pop!_OS:**
```bash
sudo apt-get update
sudo apt-get install gstreamer1.0-plugins-good gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly gstreamer1.0-libav
```

**Arch Linux / Manjaro:**
```bash
sudo pacman -S gst-plugins-base gst-plugins-good gst-plugins-bad gst-plugins-ugly gst-libav
```

**Fedora:**
*(Note: You may need to enable RPM Fusion repositories for some codecs)*
```bash
sudo dnf install gstreamer1-plugins-good gstreamer1-plugins-bad-free gstreamer1-plugins-ugly-free gstreamer1-libav
```

**OpenSUSE:**
```bash
sudo zypper install gstreamer-plugins-good gstreamer-plugins-bad gstreamer-plugins-ugly gstreamer-plugins-libav
```

Once installed, restart the application, and the loop issue should be resolved.