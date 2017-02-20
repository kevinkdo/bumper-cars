FROM scratch

ADD target/x86_64-unknown-linux-musl/release/kkdo_app /
EXPOSE 8000

CMD ["/kkdo_app"]
