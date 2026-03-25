import { gray, rgb24, Select } from "./deps.js";
import { Input } from "./deps.js";

const colors = [
  { name: "Red", value: "red", rgb: 0xff0000 },
  { name: "Dark Red", value: "dark-red", rgb: 0x8b0000 },
  { name: "Yellow", value: "yellow", rgb: 0xffd700 },
  { name: "Dark Yellow", value: "dark-yellow", rgb: 0xb8860b },
  { name: "Gray", value: "gray", rgb: 0xa9a9a9 },
  { name: "Dark Gray", value: "dark-gray", rgb: 0x696969 },
  { name: "Blue", value: "blue", rgb: 0x5b9bd5 },
  { name: "Dark Blue", value: "dark-blue", rgb: 0x00008b },
  { name: "Orange", value: "orange", rgb: 0xff8c00 },
  { name: "Dark Orange", value: "dark-orange", rgb: 0xd2691e },
];

const emojis = [
  { name: "None", value: "none" },
  { name: "\u{1F680} Rocket", value: "\u{1F680}" },
  { name: "\u{1F6A7} Construction", value: "\u{1F6A7}" },
  { name: "\u{1F9EA} Laboratory", value: "\u{1F9EA}" },
  { name: "\u{2699}\u{FE0F} Gear", value: "\u{2699}\u{FE0F}" },
  { name: "\u{1F30D} Globe", value: "\u{1F30D}" },
  { name: "\u{1F512} Lock", value: "\u{1F512}" },
  { name: "\u{26A1} Lightning", value: "\u{26A1}" },
  { name: "\u{1F4E6} Package", value: "\u{1F4E6}" },
  { name: "Custom", value: "custom" },
];

const colorMap = Object.fromEntries(colors.map((c) => [c.value, c.rgb]));

const promptColor = async () => {
  const options = [{ name: "None", value: "none" }];
  options.push(...colors.map((c) => ({
    name: `${rgb24("\u{2588}\u{2588}", c.rgb)} ${c.name}`,
    value: c.value,
  })));

  return await Select.prompt({
    message: "Select connection color",
    options,
  });
};

const promptEmoji = async () => {
  const choice = await Select.prompt({
    message: "Select connection emoji",
    options: emojis,
  });

  if (choice === "custom") {
    return await Input.prompt("Enter custom emoji");
  }

  return choice;
};

const styledName = (connection) => {
  let name = connection.name;

  if (
    connection.color && connection.color !== "none" &&
    colorMap[connection.color]
  ) {
    name = rgb24(name, colorMap[connection.color]);
  }

  const emoji = connection.emoji && connection.emoji !== "none"
    ? `${connection.emoji} `
    : "";

  return `${emoji}${name}`;
};

const formatConnectionName = (connection) => {
  return `${gray("Using connection:")} ${styledName(connection)}`;
};

export {
  colorMap,
  colors,
  emojis,
  formatConnectionName,
  promptColor,
  promptEmoji,
  styledName,
};
