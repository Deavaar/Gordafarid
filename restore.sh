#!/usr/bin/env bash
set -e

# اسکریپت بازگردانی نسخه پشتیبان پایگاه داده ربات گردآفرید
if [ -z "$1" ]; then
    echo "خطا: مسیر فایل بکاپ را وارد کنید."
    echo "مثال: ./restore.sh /home/arch/gordafarid/backups/gordafarid_backup_20261020.sql.gz"
    exit 1
fi

BACKUP_FILE="$1"

if [ ! -f "$BACKUP_FILE" ]; then
    echo "خطا: فایل مشخص‌شده یافت نشد: $BACKUP_FILE"
    exit 1
fi

echo "در حال بازگردانی پایگاه داده از: $BACKUP_FILE ..."
gunzip -c "$BACKUP_FILE" | psql -h localhost -U "${PGUSER:-gordafarid}" -d "${PGDATABASE:-gordafarid}"

echo "بازگردانی پایگاه داده با موفقیت به پایان رسید."
