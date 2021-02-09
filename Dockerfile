FROM scratch

COPY target/x86_64-unknown-linux-musl/release/potential-giggle-server /

CMD ["/potential-giggle-server"]
