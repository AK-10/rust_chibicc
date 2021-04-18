> Note:
> このリポジトリは[historical/old](https://github.com/rui314/chibicc/tree/historical/old)のブランチを参考にしています

# Dockerイメージの作成
```bash
$ docker build -t compilerbook ${dockerfileのあるdir}
```

# Dockerに入って確認したいとき
```bash
$ docker run -it -v `pwd`/:/home/user/ compilerbook /bin/bash
```

---

# テストの実行
```bash
$ docker run --rm -v `pwd`:/home/user/rust_chibicc -w /home/user/rust_chibicc compilerbook bash ./test/test.sh
```

