import { Component, ErrorInfo, ReactNode } from "react";
import { AlertTriangle, RefreshCw, Home } from "lucide-react";
import { GlassCard, GlassButton } from "@/components/ui/glass";

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
  errorInfo: ErrorInfo | null;
}

export class ErrorBoundary extends Component<Props, State> {
  public state: State = {
    hasError: false,
    error: null,
    errorInfo: null,
  };

  public static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error, errorInfo: null };
  }

  public componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error("Uncaught runtime UI error:", error, errorInfo);
    this.setState({ errorInfo });
  }

  private handleReset = () => {
    this.setState({ hasError: false, error: null, errorInfo: null });
    window.location.hash = "#/";
    window.location.reload();
  };

  public render() {
    if (this.state.hasError) {
      return (
        <div className="flex items-center justify-center min-h-[400px] p-6">
          <GlassCard className="max-w-md w-full p-6 space-y-4 border-red-500/30 bg-red-500/10">
            <div className="flex items-start gap-3">
              <AlertTriangle className="h-6 w-6 text-red-400 shrink-0 mt-0.5" />
              <div className="space-y-1">
                <h3 className="text-sm font-semibold text-white">
                  Something went wrong displaying this view
                </h3>
                <p className="text-xs text-red-200/80 font-mono break-words leading-relaxed">
                  {this.state.error?.message || "Unknown rendering exception"}
                </p>
              </div>
            </div>

            <div className="flex items-center gap-3 pt-2">
              <GlassButton variant="primary" size="sm" onClick={this.handleReset}>
                <RefreshCw className="h-3.5 w-3.5 mr-1.5" />
                Reload Page
              </GlassButton>
              <GlassButton
                variant="secondary"
                size="sm"
                onClick={() => {
                  this.setState({ hasError: false, error: null, errorInfo: null });
                  window.location.hash = "#/";
                }}
              >
                <Home className="h-3.5 w-3.5 mr-1.5" />
                Return Home
              </GlassButton>
            </div>
          </GlassCard>
        </div>
      );
    }

    return this.props.children;
  }
}
