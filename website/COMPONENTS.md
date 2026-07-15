# Components — OpenInstall Catalog

## Shared Component API

All agents MUST use these exact component names and prop signatures.

### `<Navbar />`

File: `src/components/Navbar.tsx`

Props: none (reads from context/URL)

Links:
- Logo "OpenInstall" → `/`
- "Catalog" → `/catalog`
- "Publish" → `/publish`
- "Dashboard" → `/dashboard` (if logged in)
- "Login" → `/auth/login` (if not logged in)

### `<Footer />`

File: `src/components/Footer.tsx`

Props: none

Content: copyright, GitHub link

### `<AppCard />`

File: `src/components/AppCard.tsx`

```tsx
interface AppCardProps {
  id: string;
  name: string;
  publisher: string;
  description: string;
  version: string;
}
```

Renders: bordered card with name, publisher, description, version. Links to `/app/[id]`.

### `<SearchBar />`

File: `src/components/SearchBar.tsx`

```tsx
interface SearchBarProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
}
```

Renders: input with border, triggers onChange on input.

### `<InstallButton />`

File: `src/components/InstallButton.tsx`

```tsx
interface InstallButtonProps {
  manifestUrl: string;
  appName: string;
}
```

Renders: button that copies `installer install <manifestUrl>` to clipboard.

### `<Button />`

File: `src/components/Button.tsx`

```tsx
interface ButtonProps {
  variant?: 'primary' | 'secondary';
  disabled?: boolean;
  onClick?: () => void;
  children: React.ReactNode;
  type?: 'button' | 'submit';
}
```

### `<Input />`

File: `src/components/Input.tsx`

```tsx
interface InputProps {
  type?: string;
  value: string;
  onChange: (e: React.ChangeEvent<HTMLInputElement>) => void;
  placeholder?: string;
  name?: string;
  required?: boolean;
  disabled?: boolean;
}
```

## Page Structure

Each page is a React Server Component by default. Use `"use client"` only when needed (forms, interactive state).

### Landing (`/`)
- Hero: giant "OpenInstall" text
- Subtitle: one-liner
- Install command block
- Features section (3 cards)

### Catalog (`/catalog`)
- SearchBar at top
- Grid of AppCards
- Server-side fetch from API

### App Detail (`/app/[id]`)
- App name (giant)
- Publisher, version, description
- Install command block
- Package matrix (which distros supported)

### Auth Login (`/auth/login`)
- Email + password form
- Submit button

### Auth Register (`/auth/register`)
- Email + password + confirm password form
- Submit button

### Dashboard (`/dashboard`)
- Favorites list
- Install history
- Requires auth (redirect if not logged in)

### Publish (`/publish`)
- Form: name, publisher, version, description, homepage, license
- Package URL inputs (ubuntu, arch, fedora, opensuse, fallback)
- Submit button
- Requires auth + API key

## API Routes

| Method | Endpoint | Auth | Body |
|--------|----------|------|------|
| GET | `/api/manifests` | no | — |
| POST | `/api/manifests` | API key | Manifest JSON |
| GET | `/api/manifests/[id]` | no | — |
| DELETE | `/api/manifests/[id]` | admin | — |
| POST | `/api/auth/register` | no | {email, password} |
| POST | `/api/auth/login` | no | {email, password} |
| GET | `/api/user/favorites` | JWT | — |
| POST | `/api/user/favorites` | JWT | {manifestId} |
| GET | `/api/user/history` | JWT | — |

## MongoDB Models

### User
```ts
{
  email: string (unique);
  password: string (bcrypt hash);
  role: 'user' | 'publisher' | 'admin';
  apiKey?: string;
  favorites: ObjectId[];
  createdAt: Date;
}
```

### Manifest
```ts
{
  name: string;
  publisher: string;
  version: string;
  description: string;
  homepage?: string;
  license?: string;
  packages: Record<string, string>;
  sha256?: string;
  signature?: string;
  submittedBy: ObjectId;
  downloads: number;
  createdAt: Date;
}
```

## Env Variables (for reference, NOT committed)

```
MONGODB_URI=mongodb+srv://...
JWT_SECRET=...
```
