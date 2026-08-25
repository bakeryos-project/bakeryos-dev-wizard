# Maintainer: smtdfc <smtdfc@bakeryos.smtdfc.net>
pkgname=bakeryos-dev-wizard
pkgver=1.0.0
pkgrel=1
pkgdesc="Dev Wizard for BakeryOS"
arch=('x86_64')
url="https://github.com/smtdfc/bakeryos-dev-wizard"
license=('GPL3')
depends=('gtk4' 'glib2')
makedepends=('meson' 'ninja' 'rust' 'cargo')
source=("$pkgname::git+file://$PWD")
sha256sums=('SKIP')

build() {
    arch-meson "$pkgname" build
    ninja -C build
}

package() {
    DESTDIR="$pkgdir" ninja -C build install
}
