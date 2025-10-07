import React from "react";
import "./css/loading-progress.css";

interface ChannelLoadingProgressProps {
  className?: string;
  showWhenComplete?: boolean;
}

export const ChannelLoadingProgress: React.FC<ChannelLoadingProgressProps> = () => {
  // These properties were removed from channelStore
  // Component is kept for future use but currently returns null
  return null;
};

export default ChannelLoadingProgress;
