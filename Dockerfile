FROM scratch

COPY target/x86_64-unknown-linux-musl/release/hcc-server /
COPY target/x86_64-unknown-linux-musl/release/hcc-pushover /

CMD ["/hcc-server"]
