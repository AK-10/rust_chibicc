.PHONY: test

test_docker:
	docker run --rm -v `pwd`:/home/user/rust_chibicc -w /home/user/rust_chibicc compilerbook bash ./test/test.sh

test:
	./test/test.sh
