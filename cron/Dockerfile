FROM ubuntu:16.04

# Install cron
RUN apt-get update && apt-get install -y cron postgresql postgresql-contrib

# Add files
ADD ./run_day.sh /run_day.sh
ADD ./run_min.sh /run_min.sh
ADD ./run_week.sh /run_week.sh
ADD ./entrypoint.sh /entrypoint.sh
 
RUN chmod +x /run_day.sh /run_week.sh /run_min.sh /entrypoint.sh

ENTRYPOINT /entrypoint.sh
