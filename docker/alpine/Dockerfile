FROM rustlang/rust:nightly-alpine as builder
RUN apk --update add alpine-sdk tesseract-ocr tesseract-ocr-dev leptonica-dev clang clang-libs llvm-dev leptonica
WORKDIR /usr/src/shreddr
COPY . .
ENV RUSTFLAGS='-C target-feature=-crt-static'
RUN cargo install --path .

FROM alpine
# ENV TESS_LANGUAGES "eng,deu,..." comma separated list of ISO-639 language values
ENV TESS_LANGUAGES="eng"
RUN apk add --no-cache tesseract-ocr imagemagick unpaper ocrmypdf py-pip
RUN pip3 install ocrmypdf
RUN apk del py-pip
COPY ./docker/alpine/run.sh .
RUN chmod +x run.sh
COPY --from=builder /usr/local/cargo/bin/shreddr /usr/local/bin/shreddr
ENTRYPOINT ["./run.sh"]
