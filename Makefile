PROFILE=
OBJTYPE=`uname -m`

all: bin

buildall: bin wasm win64 riscv64

release: PROFILE += --release
release: buildall

release-bin: PROFILE += --release
release-bin: bin

release-wasm: PROFILE += --release
release-wasm: wasm

release-win64: PROFILE += --release
release-win64: win64

release-riscv64: PROFILE += --release
release-riscv64: riscv64

bin:
	cargo build ${PROFILE}

install: release-bin man/man1/majestic.1
	install -m 755 target/release/majestic-lisp ${PREFIX}/bin/majestic
	install -m 644 man/man1/majestic.1 ${PREFIX}/share/man/man1/

uninstall:
	rm -f ${PREFIX}/bin/majestic
	rm -f ${PREFIX}/share/man/man1/majestic.1

test:
	cargo test

test-verbose:
	cargo test -- --nocapture

bench:
	cargo bench

run:
	cargo run ${PROFILE}

wasm:
	cargo wasi build ${PROFILE}

win64:
	cargo build ${PROFILE} --target \
		"x86_64-pc-windows-gnu" \
		--features dumb_terminal

riscv64:
	cross build ${PROFILE} --target "riscv64gc-unknown-linux-gnu"

wasmrun:
	cargo wasi run ${PROFILE}

winerun: win64
	wineconsole \
	./target/x86_64-pc-windows-gnu/debug/majestic-lisp.exe \
	2>/dev/null

present:
	@majestic -l "examples/helper.maj"

.PHONY: clean nuke

clean:
	cargo clean
	rm -rf _minted-* *.aux *.bbl *.blg *.brf *.fdb_latexmk \
		*.fls *.log *.out *.pyg *.toc *.xdv *.ilg *.ind \
		ltximg

clear:
	rm -rf target

nuke: clear clean
	rm -f *.tex *.pdf *.html *~

man/majestic(1).pdf: man/man1/majestic.1
	man -M man -Tpdf 1 majestic >"man/majestic(1).pdf"

package-objtype: release-bin man/man1/majestic.1
	mkdir -p target/tarball/${OBJTYPE}
	cp target/release/majestic-lisp target/tarball/${OBJTYPE}/
	cp man/man1/majestic.1 target/tarball/${OBJTYPE}/
	cd target/tarball/${OBJTYPE} && \
		tar -czvf ../../majestic-lisp-${OBJTYPE}.tgz .

package-wasm: release-wasm man/man1/majestic.1
	mkdir -p target/tarball/wasm32-wasi
	cp man/man1/majestic.1 target/tarball/wasm32-wasi/
	cp target/wasm32-wasi/release/majestic-lisp.wasm \
		target/tarball/wasm32-wasi/
	cd target/tarball/wasm32-wasi && \
		tar -czvf ../../majestic-lisp-wasm32-wasi.tgz .

package-win64: release-win64 man/majestic(1).pdf
	mkdir -p target/tarball/win64
	cp "man/majestic(1).pdf" target/tarball/win64/
	cp target/x86_64-pc-windows-gnu/release/majestic-lisp.exe \
		target/tarball/win64/
	cd target/tarball/win64 && \
		7z a ../../majestic-lisp-win64.zip .

package: release package-objtype package-wasm package-win64
	rm -rf target/tarball

sign: package
	gpg2 --armor --detach-sign --yes ./target/majestic-lisp-${OBJTYPE}.tgz
	gpg2 --armor --detach-sign --yes ./target/majestic-lisp-wasm32-wasi.tgz
	gpg2 --armor --detach-sign --yes ./target/majestic-lisp-win64.zip
