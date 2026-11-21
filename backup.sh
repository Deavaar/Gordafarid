#!/usr/bin/env bash
set -e

# اسکریپت پشتیبان‌گیری خودکار از پایگاه داده ربات گردآفرید
BACKUP_DIR="/home/arch/gordafarid/backups"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
FILENAME="$BACKUP_DIR/gordafarid_backup_$TIMESTAMP.sql.gz"

mkdir -p "$BACKUP_DIR"

echo "در حال ایجاد نسخه پشتیبان از پایگاه داده gordafarid..."
pg_dump -h localhost -U "${PGUSER:-gordafarid}" -d "${PGDATABASE:-gordafarid}" | gzip > "$FILENAME"

echo "نسخه پشتیبان با موفقیت در مسیر زیر ذخیره شد:"
echo "$FILENAME"

# نگهداری حداکثر ۷ نسخه پشتیبان آخر و پاک کردن موارد قدیمی‌تر
find "$BACKUP_DIR" -type f -name "gordafarid_backup_*.sql.gz" -mtime +7 -delete
