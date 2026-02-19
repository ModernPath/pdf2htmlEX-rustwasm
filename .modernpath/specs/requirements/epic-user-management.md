This document outlines the User Stories for the **User Management Epic** of the **Oxidized Document Engine (ODE)**. These stories focus on establishing a secure, scalable foundation for authentication, authorization, and the user lifecycle using the Rust/Axum/React stack.

---

## Epic: User Management
**Goal**: Provide a secure, high-performance identity layer that allows developers and administrators to manage access to the ODE transformation engine and job metadata.

---

### US-001: Secure User Registration with Argon2 Password Hashing
**As a** New Developer  
**I want to** create an account with a secure password  
**So that** I can access the ODE API and dashboard to manage my document conversion jobs.

**Acceptance Criteria**:
- [ ] User can register via a React-based form (Email, Password, Full Name).
- [ ] Backend validates email format and password strength (min 12 chars, mixed case, symbols).
- [ ] Passwords MUST be hashed using the **Argon2id** algorithm before storage in PostgreSQL.
- [ ] Duplicate email registration returns a generic "User already exists" error to prevent account enumeration.
- [ ] A new user record is created in the `users` table with a default role of `Developer`.

**Test Scenarios**:
- **Scenario 1**: Register with valid details. Expected: 201 Created, user redirected to login.
- **Scenario 2**: Register with a 6-character password. Expected: 400 Bad Request with validation error.
- **Scenario 3**: Verify DB storage. Expected: Password field contains an Argon2 hash string, not plain text.

**Affected Components**:
- `frontend/src/components/auth/RegisterForm.tsx`
- `backend/src/routes/auth.rs`
- `backend/src/services/auth_service.rs`
- `PostgreSQL`: `users` table.

**Story Estimation**:
- **Complexity**: Medium
- **Dependencies**: Database schema migration for `users` table.

---

### US-002: JWT-based Authentication and Session Management
**As a** Registered User  
**I want to** log in with my credentials and receive a secure token  
**So that** I can make authenticated requests to the Axum API without re-entering my password.

**Acceptance Criteria**:
- [ ] Login endpoint validates credentials against the PostgreSQL `users` table.
- [ ] Successful login returns a **JWT (JSON Web Token)** containing `user_id` and `role`.
- [ ] JWT is signed using a secret key stored in AWS Secret Manager (or environment variable).
- [ ] Frontend stores the JWT in a `Secure; HttpOnly; SameSite=Strict` cookie to prevent XSS/CSRF.
- [ ] A `/me` endpoint returns current user context from the token.

**Test Scenarios**:
- **Scenario 1**: Login with correct credentials. Expected: Set-Cookie header present, 200 OK.
- **Scenario 2**: Access `/api/jobs` without a token. Expected: 401 Unauthorized.
- **Scenario 3**: Login with wrong password. Expected: 401 Unauthorized.

**Affected Components**:
- `backend/src/middleware/auth_middleware.rs`
- `backend/src/handlers/login.rs`
- `frontend/src/context/AuthContext.tsx`

**Story Estimation**:
- **Complexity**: Medium
- **Dependencies**: US-001 (User Registration).

---

### US-003: Role-Based Access Control (RBAC) Middleware
**As a** System Administrator  
**I want to** restrict certain API endpoints (like system monitoring or user deletion) to specific roles  
**So that** standard developers cannot modify global system configurations.

**Acceptance Criteria**:
- [ ] Define roles: `Admin`, `Developer`, `Viewer`.
- [ ] Axum middleware extracts the role from the JWT and checks against required permissions for the route.
- [ ] `Admin` can access all routes (e.g., `/api/admin/*`).
- [ ] `Developer` can manage their own jobs and API keys.
- [ ] `Viewer` can only see job status and download converted HTML.
- [ ] Unauthorized role access returns a 403 Forbidden.

**Test Scenarios**:
- **Scenario 1**: `Developer` tries to access `/api/admin/system-stats`. Expected: 403 Forbidden.
- **Scenario 2**: `Admin` accesses the same route. Expected: 200 OK.

**Affected Components**:
- `backend/src/middleware/rbac.rs`
- `backend/src/models/role.rs`

**Story Estimation**:
- **Complexity**: Medium
- **Dependencies**: US-002 (JWT Authentication).

---

### US-004: API Key Management for Document Processing
**As a** Backend Developer  
**I want to** generate and manage API Keys  
**So that** I can integrate the ODE conversion engine into my own automated CI/CD pipelines or services.

**Acceptance Criteria**:
- [ ] User can generate a new API Key from the React Dashboard.
- [ ] API Key is shown **only once** to the user (Secret).
- [ ] Backend stores a SHA-256 hash of the API Key in the `api_keys` table.
- [ ] API Keys can be given a "Name" and an "Expiration Date".
- [ ] Users can revoke (delete) an API Key at any time.

**Test Scenarios**:
- **Scenario 1**: Generate key. Expected: Key string returned, hash stored in DB.
- **Scenario 2**: Use key in `X-API-KEY` header for conversion job. Expected: 200 OK.
- **Scenario 3**: Use revoked key. Expected: 401 Unauthorized.

**Affected Components**:
- `backend/src/handlers/api_keys.rs`
- `frontend/src/pages/Settings/ApiKeys.tsx`
- `PostgreSQL`: `api_keys` table.

**Story Estimation**:
- **Complexity**: High
- **Dependencies**: US-002 (User Auth).

---

### US-005: Password Reset Workflow (Self-Service)
**As a** User who forgot their password  
**I want to** request a password reset link via email  
**So that** I can regain access to my account securely.

**Acceptance Criteria**:
- [ ] "Forgot Password" form accepts an email address.
- [ ] System generates a short-lived (15 min), one-time-use token stored in Redis.
- [ ] An email is sent (via AWS SES or similar) containing a link to `/reset-password?token=...`.
- [ ] Reset page validates the token and allows the user to set a new password.
- [ ] Token is invalidated immediately after use.

**Test Scenarios**:
- **Scenario 1**: Request reset for non-existent email. Expected: Success message (to prevent enumeration).
- **Scenario 2**: Use expired token. Expected: Error "Link expired".
- **Scenario 3**: Successfully reset password. Expected: Old password no longer works; new one does.

**Affected Components**:
- `backend/src/services/email_service.rs`
- `backend/src/handlers/password_reset.rs`
- `Redis`: Token storage.

**Story Estimation**:
- **Complexity**: Medium
- **Dependencies**: US-001, Redis connectivity.

---

### US-006: User Profile & Account Lifecycle Management
**As a** User  
**I want to** update my profile information or delete my account  
**So that** I can keep my data current or exercise my right to be forgotten (GDPR).

**Acceptance Criteria**:
- [ ] User can update their Full Name and Email (requires re-verification if email changes).
- [ ] User can trigger "Delete Account" which performs a cascading delete of their API keys and job metadata.
- [ ] Account deletion requires password confirmation.
- [ ] UI uses Radix UI Dialogs for destructive action confirmations.

**Test Scenarios**:
- **Scenario 1**: Update name. Expected: Profile reflects change immediately.
- **Scenario 2**: Delete account. Expected: User logged out, DB records removed, cannot log in again.

**Affected Components**:
- `frontend/src/pages/Profile.tsx`
- `backend/src/handlers/user_management.rs`
- `Radix UI`: Dialog/Modal components.

**Story Estimation**:
- **Complexity**: Low
- **Dependencies**: US-001, US-002.

---

## Technical Implementation Summary

### File Reference Table (Proposed)

| Component | File Path | Action |
| :--- | :--- | :--- |
| **Backend (Rust)** | `backend/src/models/user.rs` | Create Structs for User and Claims |
| **Backend (Rust)** | `backend/src/auth/jwt.rs` | Implement Token signing/verification |
| **Backend (Rust)** | `backend/src/middleware/auth.rs` | Axum Layer for Auth validation |
| **Frontend (TS)** | `frontend/src/api/auth.ts` | Axios/Fetch wrappers for Auth endpoints |
| **Frontend (TS)** | `frontend/src/components/ProtectedRoute.tsx` | Higher-order component for route guarding |
| **Infra (Terraform)**| `terraform/rds.tf` | Ensure `users` and `api_keys` tables are provisioned |

### Verification Steps (Global)
1. [ ] **Unit Tests**: Run `cargo test` to verify password hashing and JWT logic.
2. [ ] **Integration Tests**: Use `Playwright` to simulate a full user journey (Register -> Login -> Generate API Key).
3. [ ] **Security Scan**: Run `cargo audit` to check for vulnerabilities in auth-related crates (e.g., `jsonwebtoken`, `argon2`).
4. [ ] **Accessibility**: Verify Auth forms meet WCAG 2.1 AA using `axe-core` in Playwright.