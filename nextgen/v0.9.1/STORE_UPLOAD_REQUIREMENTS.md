# BootForge Store Testing Upload Contract

BootForge v0.9.1 uses two tester-distribution channels:

1. **macOS TestFlight Store Mode**
   - App Sandbox enabled.
   - Analyzer/UI/source-validation testing only.
   - Destructive raw-disk writing is disabled in this build.
   - Full USB writing remains in the separately signed/notarized direct macOS build.

2. **Microsoft Store Private Audience**
   - Tauri Windows MSI package.
   - Intended for controlled tester distribution through Partner Center.
   - Hardware-writing behavior must still pass the sacrificial-device qualification gate.

## Apple one-time setup

Before CI can upload a macOS TestFlight build:

- Create or confirm the BootForge macOS app in App Store Connect.
- Register the exact Bundle ID that will be used by BootForge.
- The App Store Connect Bundle ID must exactly match the Tauri identifier.
- Create/download a Mac App Store Connect provisioning profile for that identifier.
- Provide the distribution signing certificate and installer distribution certificate.
- Create an App Store Connect API key with sufficient upload access.

Do **not** commit any private keys, certificates, provisioning profiles, or passwords to the repository.

Recommended GitHub Actions secrets/variables:

- `BOOTFORGE_APPLE_BUNDLE_ID`
- `BOOTFORGE_APPLE_TEAM_ID`
- `BOOTFORGE_APPLE_APP_CERT_P12_BASE64`
- `BOOTFORGE_APPLE_APP_CERT_PASSWORD`
- `BOOTFORGE_APPLE_INSTALLER_CERT_P12_BASE64`
- `BOOTFORGE_APPLE_INSTALLER_CERT_PASSWORD`
- `BOOTFORGE_APPLE_PROVISION_PROFILE_BASE64`
- `BOOTFORGE_APPLE_API_KEY_ID`
- `BOOTFORGE_APPLE_API_ISSUER_ID`
- `BOOTFORGE_APPLE_API_PRIVATE_KEY_BASE64`
- `BOOTFORGE_APPLE_APP_SIGN_IDENTITY`
- `BOOTFORGE_APPLE_INSTALLER_SIGN_IDENTITY`

The current source candidate uses `com.bobbysworld.bootforge`; treat this as provisional until the App Store Connect record confirms that exact identifier.

## Microsoft one-time setup

Before automated Store submission:

- Reserve/create BootForge in Partner Center.
- Create the first submission manually and complete the age-ratings questionnaire.
- Configure a **Private audience** tester group for controlled beta distribution.
- Associate a Microsoft Entra application with the Partner Center account and give it the required Manager access.
- Record the Partner Center product ID and Seller ID.

Recommended GitHub Actions secrets/variables:

- `BOOTFORGE_MS_TENANT_ID`
- `BOOTFORGE_MS_CLIENT_ID`
- `BOOTFORGE_MS_CLIENT_SECRET`
- `BOOTFORGE_MS_SELLER_ID`
- `BOOTFORGE_MS_PRODUCT_ID`

Never commit the client secret.

## Release rule

CI may build unsigned/preflight packages without store credentials.

CI must not claim an Apple TestFlight upload or Microsoft Store submission succeeded unless the corresponding store API returns success for the exact BootForge version/package being tested.
