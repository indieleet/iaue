patchelf --replace-needed libc.musl-aarch64.so.1 libc.so.6 ./target/release/iaue
patchelf --set-interpreter /lib/ld-linux-aarch64.so.1 ./target/release/iaue
mv ./target/release/iaue ./
