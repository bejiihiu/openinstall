import { Button } from "@/components/Button";
import { CopyCommand } from "@/components/CopyCommand";

export default function Home() {
  return (
    <main style={{ padding: "0 24px" }}>
      <div style={{ maxWidth: 1200, margin: "0 auto" }}>
        {/* Hero */}
        <section style={{ paddingTop: 96, paddingBottom: 96 }}>
          <h1
            style={{
              fontSize: "clamp(3rem, 10vw, 12rem)",
              fontWeight: 900,
              letterSpacing: "-0.05em",
              color: "#000000",
              lineHeight: 1,
              margin: 0,
            }}
          >
            OpenInstall
          </h1>
          <p
            style={{
              fontSize: 24,
              fontWeight: 400,
              color: "#000000",
              marginTop: 24,
              lineHeight: 1.6,
            }}
          >
            Linux app installer that works across distros.
          </p>
          <CopyCommand />
        </section>

        {/* Features */}
        <section style={{ paddingTop: 64, paddingBottom: 64 }}>
          <h2
            style={{
              fontSize: 48,
              fontWeight: 900,
              letterSpacing: "-0.05em",
              color: "#000000",
              margin: 0,
              marginBottom: 32,
            }}
          >
            Features
          </h2>
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(3, 1fr)",
              gap: 24,
            }}
          >
            <Card
              title="One command"
              description="Install any app with a single command. No more six download buttons."
            />
            <Card
              title="Any distro"
              description="Ubuntu, Arch, Fedora, openSUSE — we handle the differences."
            />
            <Card
              title="Verified"
              description="SHA256 + Ed25519 signature verification. Know what you install."
            />
          </div>
        </section>

        {/* How it works */}
        <section style={{ paddingTop: 64, paddingBottom: 64 }}>
          <h2
            style={{
              fontSize: 48,
              fontWeight: 900,
              letterSpacing: "-0.05em",
              color: "#000000",
              margin: 0,
              marginBottom: 32,
            }}
          >
            How it works
          </h2>
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(3, 1fr)",
              gap: 24,
            }}
          >
            <Step
              number="01"
              text="Developer publishes a manifest"
            />
            <Step
              number="02"
              text="User runs one install command"
            />
            <Step
              number="03"
              text="OpenInstall handles the rest"
            />
          </div>
        </section>

        {/* CTA */}
        <section
          style={{
            paddingTop: 64,
            paddingBottom: 128,
            display: "flex",
            alignItems: "center",
            gap: 24,
          }}
        >
          <h2
            style={{
              fontSize: 48,
              fontWeight: 900,
              letterSpacing: "-0.05em",
              color: "#000000",
              margin: 0,
            }}
          >
            Get started
          </h2>
          <Button variant="primary">Install now</Button>
          <Button variant="secondary">Browse apps</Button>
        </section>
      </div>
    </main>
  );
}

function Card({ title, description }: { title: string; description: string }) {
  return (
    <div
      style={{
        backgroundColor: "#FFFFFF",
        border: "1px solid #000000",
        padding: 32,
        borderRadius: 0,
      }}
    >
      <h3
        style={{
          fontSize: 24,
          fontWeight: 700,
          color: "#000000",
          margin: 0,
          marginBottom: 16,
        }}
      >
        {title}
      </h3>
      <p
        style={{
          fontSize: 16,
          fontWeight: 400,
          color: "#000000",
          lineHeight: 1.6,
          margin: 0,
        }}
      >
        {description}
      </p>
    </div>
  );
}

function Step({ number, text }: { number: string; text: string }) {
  return (
    <div
      style={{
        borderLeft: "2px solid #000000",
        paddingLeft: 24,
      }}
    >
      <span
        style={{
          fontSize: 64,
          fontWeight: 900,
          color: "#000000",
          lineHeight: 1,
          display: "block",
          marginBottom: 8,
        }}
      >
        {number}
      </span>
      <p
        style={{
          fontSize: 16,
          fontWeight: 400,
          color: "#000000",
          lineHeight: 1.6,
          margin: 0,
        }}
      >
        {text}
      </p>
    </div>
  );
}
