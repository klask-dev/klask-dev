# Testing the Klask Application

This guide walks you through testing the authentication system and overall functionality.

## Quick Start

### Automated Setup (Recommended)
```bash
# From the project root
./start-dev.sh
```

This script will:
- Check prerequisites (Rust, Node.js, Docker)
- Start PostgreSQL with Docker
- Set up environment files
- Start both backend and frontend
- Provide testing instructions

### Manual Setup

1. **Start Database**:
   ```bash
   cd klask-rs
   docker-compose -f docker-compose.yml up -d
   ```

2. **Start Backend**:
   ```bash
   cd klask-rs
   cp .env.example .env
   cargo run
   ```

3. **Start Frontend** (new terminal):
   ```bash
   cd klask-react
   cp .env.example .env
   npm install
   npm run dev
   ```

## Testing Scenarios

### 1. Health Check ✅
**Verify both services are running**

- **Backend**: http://localhost:8080/health
  - Should return: `{"status":"healthy"}`
- **Frontend**: http://localhost:5173
  - Should show login page

### 2. User Registration 🔐
**Test account creation**

1. Navigate to: http://localhost:5173/register
2. Fill the form:
   ```
   Username: testuser
   First Name: Test
   Last Name: User
   Email: test@example.com
   Password: TestPass123
   Confirm Password: TestPass123
   ```
3. Click "Create account"
4. **Expected**: Redirect to search page with success message

**Verify in Database**:
```sql
-- Connect to PostgreSQL
psql -h localhost -U klask_user -d klask_dev

-- Check user was created
SELECT username, email, role, active, created_at FROM users;
```

### 3. User Login 🔑
**Test authentication**

1. Navigate to: http://localhost:5173/login
2. Enter credentials:
   ```
   Username: testuser
   Password: TestPass123
   ```
3. Click "Sign in"
4. **Expected**: Redirect to search page

### 4. Protected Routes 🛡️
**Test route protection**

1. After login, you should be on `/search`
2. Try navigating to `/admin` (should redirect if not admin)
3. Logout and try accessing `/search` directly (should redirect to login)

### 5. Form Validation 📝
**Test client-side validation**

**Registration Form**:
- Try weak password: `123` → Should show validation error
- Try mismatched passwords → Should show error
- Try invalid email → Should show error
- Try duplicate username → Should show server error

**Login Form**:
- Try empty fields → Should show validation errors
- Try invalid credentials → Should show authentication error

### 6. API Integration 🔌
**Test backend communication**

Open browser DevTools (F12) and check:

1. **Network Tab**: API calls to `/api/auth/register` and `/api/auth/login`
2. **Console**: No JavaScript errors
3. **Application Tab**: JWT token stored in localStorage
4. **Headers**: Authorization header included in authenticated requests

### 7. State Management 💾
**Test persistent authentication**

1. Login successfully
2. Refresh the page → Should stay logged in
3. Close browser and reopen → Should stay logged in
4. Logout → Should clear state and redirect

## Debugging Common Issues

### Backend Issues

**Database Connection Failed**:
```bash
# Check if PostgreSQL is running
docker ps | grep postgres

# Check connection
psql -h localhost -U klask_user -d klask_dev -c "SELECT 1;"
```

**Port 8080 Already in Use**:
```bash
# Find process using port
lsof -i :8080

# Kill process (replace PID)
kill -9 <PID>
```

**Migration Errors**:
```bash
# Install sqlx-cli if not installed
cargo install sqlx-cli

# Run migrations manually
cd klask-rs
sqlx migrate run
```

### Frontend Issues

**API Connection Failed**:
1. Check backend is running on port 8080
2. Verify VITE_API_BASE_URL in `.env`
3. Check CORS settings in backend

**TypeScript Errors**:
```bash
cd klask-react
npm run build  # Check for compilation errors
```

**Dependency Issues**:
```bash
cd klask-react
rm -rf node_modules package-lock.json
npm install
```

### Authentication Issues

**JWT Token Problems**:
```javascript
// In browser console
localStorage.getItem('klask-auth')  // Check stored auth state
localStorage.clear()  // Clear if corrupted
```

**Password Hashing Issues**:
```sql
-- Check if passwords are hashed in database
SELECT username, password_hash FROM users;
-- Should NOT show plain text passwords
```

## Expected Behavior

### ✅ Working Features
- [x] User registration with validation
- [x] User login with JWT tokens
- [x] Form validation (client & server-side)
- [x] Route protection
- [x] Persistent login state
- [x] Password hashing (Argon2)
- [x] Error handling and display

### 🚧 In Development
- [ ] Search functionality
- [ ] Repository management
- [ ] File browsing
- [ ] Admin features

### 📊 Test Metrics
- **Frontend**: TypeScript compilation ✅
- **Backend**: Rust compilation ✅
- **Database**: Migrations ✅
- **Authentication**: JWT flow ✅
- **Validation**: Form & API ✅

## Next Steps

Once authentication is working:

1. **Search Interface**: Test search functionality
2. **Repository Management**: Add/crawl repositories
3. **File Browsing**: View indexed files
4. **Admin Features**: User management

## Support

If you encounter issues:

1. Check the [DEVELOPMENT.md](./DEVELOPMENT.md) guide
2. Verify all prerequisites are installed
3. Check server logs for detailed error messages:
   ```bash
   # Backend logs
   cd klask-rs
   RUST_LOG=debug cargo run
   
   # Frontend logs
   Open browser DevTools → Console
   ```

Happy testing! 🧪