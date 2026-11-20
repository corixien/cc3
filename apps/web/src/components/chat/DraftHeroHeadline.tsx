// CC3: empty-chat hero. A plain greeting that cycles through a few lines; nothing clickable.
import { useEffect, useState } from "react";

const GREETINGS = [
  "What should we work on?",
  "What's up?",
  "Ready when you are.",
  "What are we building today?",
  "How can I help?",
  "Where should we start?",
] as const;

const CYCLE_MS = 6000;

export function DraftHeroHeadline() {
  const [index, setIndex] = useState(0);
  useEffect(() => {
    const timer = window.setInterval(
      () => setIndex((current) => (current + 1) % GREETINGS.length),
      CYCLE_MS,
    );
    return () => window.clearInterval(timer);
  }, []);
  return (
    <h1
      key={index}
      className="fade-in animate-in w-full text-center font-normal text-3xl text-foreground tracking-tight duration-700 sm:text-5xl"
    >
      {GREETINGS[index]}
    </h1>
  );
}
