import { Command } from "https://deno.land/x/cliffy@v0.25.2/command/mod.ts";
import { Input } from "https://deno.land/x/cliffy@v0.25.2/prompt/input.ts";
import { brightBlue, brightYellow, brightRed, bold } from "https://deno.land/std@0.158.0/fmt/colors.ts";
import * as log from "https://deno.land/std@0.158.0/log/mod.ts";
import { BaseHandler } from "https://deno.land/std@0.158.0/log/handlers.ts?s=BaseHandler";
import { LogLevels } from "https://deno.land/std@0.158.0/log/mod.ts";

const colors = {
  brightBlue,
  brightYellow,
  brightRed,
  bold
}

export { Command, Input, colors, log, LogLevels, BaseHandler };
