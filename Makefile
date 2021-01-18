.PHONY: all debug release

debug:
	cargo build --all

release:
	cargo build --release --all
	cargo build --package=node-bft --release

all: debug release

testdata:
	for d in 100 500 1000 2000 3000 4000 5000 ; do \
		mkdir -p test/d$$d-n4 ; \
		./target/release/genconfig -n 4 -d $$d -b 1 -C 15000 -P 16000 -t ./test/d$$d-n4 ; \
	done
	
