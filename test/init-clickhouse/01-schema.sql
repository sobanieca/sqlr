CREATE DATABASE IF NOT EXISTS world;

CREATE TABLE IF NOT EXISTS world.city (
  id UInt32,
  name String,
  country_code String,
  district String,
  population UInt32
) ENGINE = MergeTree()
ORDER BY id;

CREATE TABLE IF NOT EXISTS world.country (
  code String,
  name String,
  continent String,
  region String,
  surface_area Float32,
  indep_year Nullable(Int16),
  population UInt32,
  life_expectancy Nullable(Float32),
  gnp Nullable(Decimal(10,2)),
  gnp_old Nullable(Decimal(10,2)),
  local_name String,
  government_form String,
  head_of_state Nullable(String),
  capital Nullable(UInt32),
  code2 String
) ENGINE = MergeTree()
ORDER BY code;

CREATE TABLE IF NOT EXISTS world.country_language (
  country_code String,
  language String,
  is_official UInt8,
  percentage Float32
) ENGINE = MergeTree()
ORDER BY (country_code, language);

CREATE TABLE IF NOT EXISTS world.country_flag (
  code2 String,
  emoji String,
  unicode Nullable(String)
) ENGINE = MergeTree()
ORDER BY code2;

INSERT INTO world.city (id, name, country_code, district, population) VALUES
(129, 'Oranjestad', 'ABW', 'Aruba', 29034),
(1, 'Kabul', 'AFG', 'Kabol', 1780000),
(56, 'Luanda', 'AGO', 'Luanda', 2022000);

INSERT INTO world.country (code, name, continent, region, surface_area, indep_year, population, life_expectancy, gnp, gnp_old, local_name, government_form, head_of_state, capital, code2) VALUES
('ABW', 'Aruba', 'North America', 'Caribbean', 193, 1986, 103000, 78.4, 828.00, 793.00, 'Aruba', 'Nonmetropolitan Territory of The Netherlands', 'Willem-Alexander', 129, 'AW'),
('AFG', 'Afghanistan', 'Asia', 'Southern and Central Asia', 652090, 1919, 22720000, 45.9, 5976.00, NULL, 'Afganistan/Afqanestan', 'Islamic Emirate', 'Mohammad Omar', 1, 'AF'),
('AGO', 'Angola', 'Africa', 'Central Africa', 1246700, 1975, 12878000, 38.3, 6648.00, 7984.00, 'Angola', 'Republic', 'Jose Eduardo dos Santos', 56, 'AO');

INSERT INTO world.country_language (country_code, language, is_official, percentage) VALUES
('ABW', 'Dutch', 1, 5.3),
('ABW', 'English', 0, 9.5),
('ABW', 'Papiamento', 0, 76.7),
('ABW', 'Spanish', 0, 7.4),
('AFG', 'Balochi', 0, 0.9),
('AFG', 'Dari', 1, 32.1),
('AFG', 'Pashto', 1, 52.4),
('AFG', 'Turkmenian', 0, 1.9),
('AFG', 'Uzbek', 0, 8.9),
('AGO', 'Kimbundu', 0, 25.0),
('AGO', 'Mbundu', 0, 22.0),
('AGO', 'Ovimbundu', 0, 37.0);

INSERT INTO world.country_flag (code2, emoji, unicode) VALUES
('AW', '🇦🇼', 'U+1F1E6 U+1F1FC'),
('AF', '🇦🇫', 'U+1F1E6 U+1F1EB'),
('AO', '🇦🇴', 'U+1F1E6 U+1F1F4');
