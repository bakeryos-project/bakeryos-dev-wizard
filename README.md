# BakeryOS Dev Wizard

**BakeryOS Dev Wizard** (`org.bakeryos.devwizard`) is a post-installation setup wizard and developer environment installer for **BakeryOS**. Built with **Rust**, **GTK 4**, and **Libadwaita**, it provides a modern, clean, and intuitive user interface to help developers easily configure their system and install development tools, runtimes, and applications.

---

## 🚀 Features

- **Developer Tool Management**: Easily browse and install essential developer tools, SDKs, IDEs, and utilities.
- **Multiple Installation Backends**:
  - **Flathub**: Install Flatpak applications directly from Flathub.
  - **Pacman**: Install system packages via Pacman with privilege elevation (`pkexec`).
  - **Custom Scripts**: Run tailored post-install shell setup routines.
  - **Interactive Terminal Setup**: Launch interactive setup steps in GNOME Console (`kgx`).
- **Modern GNOME UI**: Built natively with GTK 4 and Libadwaita following GNOME HIG (Human Interface Guidelines).

---

## 🛠️ Built With

- **Language**: [Rust](https://www.rust-lang.org/)
- **UI Framework**: [GTK 4](https://gtk.org/) & [Libadwaita](https://gnome.pages.gitlab.gnome.org/libadwaita/)
- **Build System**: [Meson](https://mesonbuild.com/) & Cargo

---

## 📋 Prerequisites

Before building from source, ensure you have the following dependencies installed on your system:

- **Rust** (stable toolchain)
- **Meson** (`>= 1.0.0`) & **Ninja**
- **GTK 4** (`>= 4.14`) and **Libadwaita** (`>= 1.6`)
- `gettext` tools for translations

---

## 🏗️ Building and Running

```bash
# Setup build directory
meson setup builddir

# Compile the project
meson compile -C builddir

# Run the compiled executable directly
./builddir/src/bakeryos-dev-wizard
```

Installing System-wide

```bash
# Install to system (default prefix is /usr/local)
sudo meson install -C builddir
```
