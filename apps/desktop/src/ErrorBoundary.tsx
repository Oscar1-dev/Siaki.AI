import { Component, type ErrorInfo, type ReactNode } from "react";
import { writeFrontendLog } from "./ipc";

interface Props { readonly children: ReactNode }
interface State { readonly error: Error | null }

export class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State { return { error }; }

  componentDidCatch(error: Error, info: ErrorInfo): void {
    void writeFrontendLog({
      level: "error",
      event: "renderer.error_boundary",
      message: error.message.slice(0, 500) || "Unknown renderer error",
      context: { componentStackPresent: Boolean(info.componentStack) },
    }).catch(() => undefined);
  }

  render(): ReactNode {
    if (this.state.error) {
      return <main className="fatal" role="alert"><p>SAIKI encountered a renderer error.</p><button onClick={() => window.location.reload()}>Reload shell</button></main>;
    }
    return this.props.children;
  }
}
