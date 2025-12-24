# ⚠️ PASSWORD UPDATE REMINDER

## Action Required: Update Default Passwords

**Current Status**: Default development passwords are in use.

**Location**: Check `.password-reminder` file in the project root for details.

**When to Update**:
- ✅ Before any production deployment
- ✅ Before sharing repository publicly  
- ✅ Before using in public-facing environments
- ✅ Before connecting to the internet

**Quick Update Steps**:
1. Edit `.env` file
2. Change `GF_SECURITY_ADMIN_PASSWORD` to a strong password
3. Change `POSTGRES_PASSWORD` to a strong password  
4. Update `DATABASE_URL` with new PostgreSQL password
5. Restart services: `docker-compose restart`

**Password Requirements**:
- Minimum 16 characters
- Mix of uppercase, lowercase, numbers, symbols
- Unique (not reused)
- Use a password manager

---

**Note**: This file is committed to git as a reminder. The actual passwords are in `.env` which is gitignored.

