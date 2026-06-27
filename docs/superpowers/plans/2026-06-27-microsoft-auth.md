# Microsoft Auth Replacement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace Facebook OAuth with Microsoft Entra ID (personal accounts) — same non-admin, firstname/lastname/email flow.

**Architecture:** Remove Facebook provider enum variant, config fields, and auth service function. Add Microsoft variant with equivalent OAuth flow using raw HTTP calls to Microsoft Identity Platform v2.0 and Microsoft Graph API. Update DB constraint. Swap frontend button and provider type. Update CI/CD env vars.

**Tech Stack:** Rust (reqwest, axum), TypeScript/Vue 3, PostgreSQL, Terraform, GitHub Actions

---

### Task 1: Backend Config — Remove Facebook, Add Microsoft

**Files:**
- Modify: `back/src/config.rs`
- Modify: `back/src/auth/model.rs`

- [ ] **Step 1: Update config.rs — swap env var fields**

In `back/src/config.rs`, replace the Facebook fields and their deserialization with Microsoft equivalents:

```rust
// Remove these:
pub facebook_client_id: String,
pub facebook_client_secret: String,

// Add these:
pub microsoft_client_id: String,
pub microsoft_client_secret: String,
```

- [ ] **Step 2: Update model.rs — swap Provider enum**

In `back/src/auth/model.rs`, replace the `Facebook` variant with `Microsoft` in the `Provider` enum:

```rust
pub enum Provider {
    Google,
    Microsoft,
}
```

Update the `FromStr` and `Display` impls accordingly — parse `"microsoft"` to `Provider::Microsoft`, display `Provider::Microsoft` as `"microsoft"`.

- [ ] **Step 3: Run backend compilation check**

Run: `cargo build --all-features`
Expected: Build succeeds (may fail if other code still references Facebook — that's expected, subsequent tasks fix that)

---

### Task 2: Backend Auth Service — Add Microsoft exchange, Remove Facebook

**Files:**
- Modify: `back/src/auth/service.rs`

- [ ] **Step 1: Add `MicrosoftTokenResponse` and `MicrosoftUser` structs**

Near the top of `service.rs`, add response structs for Microsoft's token and Graph API:

```rust
#[derive(Deserialize, Debug)]
pub struct MicrosoftTokenResponse {
    pub access_token: String,
}

#[derive(Deserialize, Debug)]
pub struct MicrosoftUser {
    #[serde(default)]
    pub given_name: Option<String>,
    #[serde(default)]
    pub surname: Option<String>,
    #[serde(default)]
    pub mail: Option<String>,
}
```

- [ ] **Step 2: Write `microsoft_exchange_and_email()` function**

Add this function alongside (or replacing) `facebook_exchange_and_email()`:

```rust
pub async fn microsoft_exchange_and_email(
    http_client: &Client,
    config: &Config,
    code: &str,
) -> Result<ProviderUser> {
    let token_url = format!(
        "https://login.microsoftonline.com/consumers/oauth2/v2.0/token"
    );
    let params = [
        ("client_id", &config.microsoft_client_id),
        ("client_secret", &config.microsoft_client_secret),
        ("code", &code.to_string()),
        ("redirect_uri", &format!("{}/auth/microsoft/callback", config.app_base_url)),
        ("grant_type", &"authorization_code".to_string()),
    ];
    let token_resp = http_client
        .post(&token_url)
        .form(&params)
        .send()
        .await
        .context("microsoft token exchange request failed")?
        .json::<MicrosoftTokenResponse>()
        .await
        .context("microsoft token exchange response parse failed")?;

    let user_url = "https://graph.microsoft.com/v1.0/me?$select=displayName,givenName,surname,mail";
    let user_resp = http_client
        .get(user_url)
        .bearer_auth(&token_resp.access_token)
        .send()
        .await
        .context("microsoft graph userinfo request failed")?
        .json::<MicrosoftUser>()
        .await
        .context("microsoft graph userinfo response parse failed")?;

    let email = user_resp
        .mail
        .ok_or_else(|| anyhow!("microsoft did not return email"))?;

    Ok(ProviderUser {
        email: email.to_lowercase(),
        first_name: user_resp.given_name,
        last_name: user_resp.surname,
    })
}
```

- [ ] **Step 3: Remove Facebook structs and function**

Remove from `service.rs`:
- `FacebookTokenResponse` struct
- `FacebookUser` struct
- `facebook_exchange_and_email()` function
- `facebook_start()` function
- `facebook_start_or_error()` function (if exists separately)

- [ ] **Step 4: Update `oauth_callback()` to dispatch to Microsoft**

In `service.rs`, in the `oauth_callback()` function, replace the `"facebook"` branch with `"microsoft"`:

```rust
"microsoft" => {
    let provider_user = microsoft_exchange_and_email(&http_client, config, &code).await?;
    ...
}
```

Remove the `"facebook"` branch.

- [ ] **Step 5: Run backend compilation check**

Run: `cargo build --all-features`
Expected: Build succeeds

---

### Task 3: Database Migration

**Files:**
- Create: `back/migrations/0025_microsoft_auth.sql`

- [ ] **Step 1: Create migration file**

```sql
-- Replace facebook with microsoft in provider check constraint
ALTER TABLE users DROP CONSTRAINT IF EXISTS users_provider_check;
ALTER TABLE users ADD CONSTRAINT users_provider_check CHECK (provider IN ('google', 'microsoft'));
```

- [ ] **Step 2: Run migrations to verify**

Run: `cargo test --all-features`
Expected: Tests pass (migration runs as part of test setup)

---

### Task 4: Frontend — Login Page and Session Type

**Files:**
- Modify: `front/src/auth/LoginPage.vue`
- Modify: `front/src/auth/session.ts`

- [ ] **Step 1: Update `session.ts` Provider type**

Change `Provider` type from `"google" | "facebook"` to `"google" | "microsoft"`.

- [ ] **Step 2: Update `LoginPage.vue` — swap Facebook for Microsoft button**

Replace the Facebook button block:

```vue
<a class="login-button" :href="providerStartUrl('microsoft', redirectPath)">
  <MicrosoftLogo class="login-button-icon" />
  Continue with Microsoft
</a>
```

Provide an inline Microsoft SVG logo (simple "M" or Microsoft symbol) as `MicrosoftLogo` component, or use an SVG directly.

- [ ] **Step 3: Run frontend lint and tests**

Run: `cd front && npm run lint && npm run test`
Expected: All pass

---

### Task 5: Backend Tests — Update Auth Tests

**Files:**
- Find and modify auth-related tests in `back/src/auth/`

- [ ] **Step 1: Find and update tests referencing Facebook**

Search for any tests referencing `"facebook"` in `back/src/auth/` and update them to use `"microsoft"` instead.

Run: `cargo test --all-features`
Expected: All tests pass

---

### Task 6: Infrastructure — Update CI/CD Env Vars

**Files:**
- Modify: `.github/workflows/infrastructure-ci.yml`
- Modify: `.github/workflows/infrastructure-apply.yml`
- Modify: `infrastructure/terraform.tfvars.example`
- Modify: `infrastructure/README.md`
- Modify: `README.md`

- [ ] **Step 1: Update CI workflows**

In both `.github/workflows/infrastructure-ci.yml` and `.github/workflows/infrastructure-apply.yml`, replace:

```yaml
"FACEBOOK_CLIENT_ID": "${{ secrets.FACEBOOK_CLIENT_ID }}",
"FACEBOOK_CLIENT_SECRET": "${{ secrets.FACEBOOK_CLIENT_SECRET }}",
```

with:

```yaml
"MICROSOFT_CLIENT_ID": "${{ secrets.MICROSOFT_CLIENT_ID }}",
"MICROSOFT_CLIENT_SECRET": "${{ secrets.MICROSOFT_CLIENT_SECRET }}",
```

- [ ] **Step 2: Update terraform.tfvars.example**

Replace Facebook entries with Microsoft entries in the commented example block.

- [ ] **Step 3: Update infrastructure/README.md**

Replace `FACEBOOK_CLIENT_ID`, `FACEBOOK_CLIENT_SECRET` with `MICROSOFT_CLIENT_ID`, `MICROSOFT_CLIENT_SECRET` in the OAuth/runtime app secrets table.

- [ ] **Step 4: Update root README.md**

Replace the Facebook rows with Microsoft rows in the environment variables table.

---

### Task 7: Environment Files

**Files:**
- Modify: `back/.env`

- [ ] **Step 1: Update back/.env**

Replace:
```
FACEBOOK_CLIENT_ID=fake
FACEBOOK_CLIENT_SECRET=fake
```
with:
```
MICROSOFT_CLIENT_ID=fake
MICROSOFT_CLIENT_SECRET=fake
```

---

### Task 8: Final Verification

- [ ] **Step 1: Backend build**

Run: `cargo build --all-features`
Expected: Build succeeds with no warnings about dead code or unused imports

- [ ] **Step 2: Backend tests**

Run: `cargo test --all-features`
Expected: All tests pass

- [ ] **Step 3: Frontend build**

Run: `cd front && npm run build`
Expected: Build succeeds

- [ ] **Step 4: Frontend lint**

Run: `cd front && npm run lint`
Expected: No lint errors

- [ ] **Step 5: Frontend tests**

Run: `cd front && npm run test`
Expected: All tests pass

- [ ] **Step 6: No remaining Facebook references**

Run: `rg -i "facebook" --type rust --type typescript --type vue --type yaml --type sql`
Expected: Zero results in source code (false positives in unrelated text like event descriptions are OK)

---

### Post-Implementation Checklist

- [ ] `MICROSOFT_CLIENT_ID` and `MICROSOFT_CLIENT_SECRET` added as GitHub Actions secrets
- [ ] Azure Portal app registration created with personal-MSA-only and redirect URI configured
- [ ] `FACEBOOK_CLIENT_ID` and `FACEBOOK_CLIENT_SECRET` removed from GitHub secrets (optional cleanup)
