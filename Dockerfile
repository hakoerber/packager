FROM docker.io/library/alpine:3

RUN apk add --no-cache tini

COPY target/x86_64-unknown-linux-musl/release/packager /usr/local/bin/packager

ENTRYPOINT ["tini", "--"]

CMD [ \
    "/usr/local/bin/packager", \
    "--database-url", "/var/lib/packager/db/db.sqlite", \
    "serve", \
    "--bind", "0.0.0.0", \
    "--port", "3000" \
]
