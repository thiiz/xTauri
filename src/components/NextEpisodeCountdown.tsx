import { useEffect, useState } from "react";

interface NextEpisodeCountdownProps {
  /** Tempo restante em segundos */
  timeRemaining: number;
  /** Título do próximo episódio */
  nextEpisodeTitle: string;
  /** Número do próximo episódio */
  nextEpisodeNumber: string;
  /** Callback quando o usuário confirma */
  onConfirm: () => void;
  /** Callback quando o usuário cancela */
  onCancel: () => void;
  /** Se deve mostrar o contador */
  show: boolean;
}

export default function NextEpisodeCountdown({
  timeRemaining,
  nextEpisodeTitle,
  nextEpisodeNumber,
  onConfirm,
  onCancel,
  show
}: NextEpisodeCountdownProps) {
  const [countdown, setCountdown] = useState(timeRemaining);

  useEffect(() => {
    if (!show) return;

    setCountdown(timeRemaining);

    const interval = setInterval(() => {
      setCountdown((prev) => {
        if (prev <= 1) {
          clearInterval(interval);
          onConfirm(); // Auto-avança quando o tempo acaba
          return 0;
        }
        return prev - 1;
      });
    }, 1000);

    return () => clearInterval(interval);
  }, [show, timeRemaining, onConfirm]);

  if (!show) return null;

  return (
    <div className="next-episode-countdown">
      <div className="countdown-content">
        <div className="countdown-header">
          <h3>Próximo Episódio</h3>
          <div className="countdown-timer">{countdown}s</div>
        </div>

        <div className="countdown-info">
          <div className="episode-number">Episódio {nextEpisodeNumber}</div>
          <div className="episode-title">{nextEpisodeTitle}</div>
        </div>

        <div className="countdown-actions">
          <button className="countdown-btn countdown-btn-confirm" onClick={onConfirm}>
            <span>▶</span> Reproduzir Agora
          </button>
          <button className="countdown-btn countdown-btn-cancel" onClick={onCancel}>
            Cancelar
          </button>
        </div>

        <div className="countdown-progress">
          <div
            className="countdown-progress-bar"
            style={{ width: `${(countdown / timeRemaining) * 100}%` }}
          />
        </div>
      </div>
    </div>
  );
}
