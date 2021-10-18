FROM debian:11

RUN apt-get update && apt-get install -y \
  tstools

CMD ["tsplay", "/mnt/dumps/1.ts", "239.255.1.1:5500", "-loop"]
# CMD ["tsplay", "/mnt/dumps/2.ts", "239.255.1.1:5500", "-loop"]
# CMD ["tsplay", "/mnt/dumps/3.ts", "239.255.1.1:5500", "-loop"]
