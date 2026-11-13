// `npm run stop`. Use this when the terminal that ran `npm run dev` is already gone.
//
// Process: ask scripts/port.mjs to free 43123, but only if the listener is
// this repo's Vite or scripts/dev.mjs. A different program is left running,
// and its pid is printed. Do not kill a pid you do not recognise.

import { PORT, formatHolder, releaseOurs } from "./port.mjs";

const result = await releaseOurs();
if (!result.ok) {
  console.error(`Port ${PORT} is still in use by another program.`);
  const holders = result.foreign.length ? result.foreign : [];
  if (holders.length === 0) {
    console.error("The listener could not be identified, so nothing was killed.");
  }
  for (const holder of holders) console.error(`  ${formatHolder(holder)}`);
  process.exit(1);
}
console.log(`Port ${PORT} is free.`);
