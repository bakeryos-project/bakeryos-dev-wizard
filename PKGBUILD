# Maintainer: smtdfc <me.smtdfc@gmail.com>

pkgname=bakeryos-dev-wizard
pkgver=0.1.2
pkgrel=1
pkgdesc="A post-installation setup wizard and developer environment installer for BakeryOS."
arch=('x86_64')
url="https://github.com/bakeryos-project/bakeryos-dev-wizard"
license=('GPL-3.0-or-later')
depends=('gtk4' 'libadwaita' 'glib2')
makedepends=('base-devel' 'meson' 'rust' 'cargo' 'blueprint-compiler' 'clang' 'lld' 'gcc')
source=()
sha256sums=()

build() {
    cd $startdir

    export CFLAGS=""
    export CXXFLAGS=""
    export LDFLAGS=""
    export RUSTFLAGS=""

    arch-meson . build
    meson compile -C build
}

package() {
    cd $startdir
    meson install -C build --no-rebuild --destdir "$pkgdir"
}
