.PHONY: test
docker_build:
	docekr build . -t compilerbook:latest

test_docker:
	docker run --rm -it -v `pwd`:/home/user/rust_chibicc -w /home/user/rust_chibicc compilerbook make test

test:
	cargo run --release test.c > tmp.s
	echo 'int char_fn() { return 257; } int static_fn() { return 5; }' | \
		gcc -xc -c -o tmp2.o -
	gcc -static -o tmp tmp.s tmp2.o
	./tmp

output:
	cargo run --release test.c

nqueen:
	cargo run --release examples/nqueen.c > tmp.s
	gcc -static -o tmp tmp.s
	./tmp

