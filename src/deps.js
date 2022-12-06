import { Command } from "https://deno.land/x/cliffy@v0.25.4/command/mod.ts";
import { Input } from "https://deno.land/x/cliffy@v0.25.4/prompt/input.ts";
import { Select } from "https://deno.land/x/cliffy@v0.25.4/prompt/select.ts";
import { Toggle } from "https://deno.land/x/cliffy@v0.25.4/prompt/toggle.ts";
import { gray, white, brightYellow, brightRed, bold } from "https://deno.land/std@0.158.0/fmt/colors.ts";
import * as log from "https://deno.land/std@0.158.0/log/mod.ts";
import { BaseHandler } from "https://deno.land/std@0.158.0/log/handlers.ts?s=BaseHandler";
import { LogLevels } from "https://deno.land/std@0.158.0/log/mod.ts";
import { Table } from "https://deno.land/x/cliffy@v0.25.5/table/mod.ts";

const colors = {
  gray,
  white,
  brightYellow,
  brightRed,
  bold
}

export { Command, Input, Select, Toggle, colors, log, LogLevels, BaseHandler, Table };
