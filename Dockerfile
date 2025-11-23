FROM rust:1.91-alpine as builder
WORKDIR /usr/src/ucsc_menu
COPY . .
RUN apk add --no-cache musl-dev
# RUN apt update && apt install -y git pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
# RUN RUSTFLAGS="-Ctarget-feature=-crt-static" cargo install --path . 
RUN RUSTFLAGS="-Ctarget-feature=-crt-static" cargo install --locked --path .

FROM alpine:latest
RUN apk add --no-cache musl libgcc
ENV PORT 8080
ENV HOST 0.0.0.0
# ENV GOOGLE_APPLICATION_CREDENTIALS /usr/src/ucsc_menu/ucsc-menu-firebase-adminsdk-6etok-1eebb222da.json
COPY --from=builder /usr/local/cargo/bin/ucsc_menu /usr/local/bin/ucsc_menu
# COPY --from=builder /usr/src/ucsc_menu/ucsc-menu-firebase-adminsdk-6etok-1eebb222da.json /usr/src/ucsc_menu/ucsc-menu-firebase-adminsdk-6etok-1eebb222da.json
EXPOSE 8080
CMD ["ucsc_menu"]