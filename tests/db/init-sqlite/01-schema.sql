CREATE TABLE IF NOT EXISTS city (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  country_code TEXT NOT NULL,
  district TEXT NOT NULL,
  population INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS country (
  code TEXT NOT NULL PRIMARY KEY,
  name TEXT NOT NULL,
  continent TEXT NOT NULL DEFAULT 'Asia',
  region TEXT NOT NULL,
  surface_area REAL NOT NULL DEFAULT 0,
  indep_year INTEGER DEFAULT NULL,
  population INTEGER NOT NULL DEFAULT 0,
  life_expectancy REAL DEFAULT NULL,
  gnp REAL DEFAULT NULL,
  gnp_old REAL DEFAULT NULL,
  local_name TEXT NOT NULL,
  government_form TEXT NOT NULL,
  head_of_state TEXT DEFAULT NULL,
  capital INTEGER DEFAULT NULL,
  code2 TEXT NOT NULL,
  FOREIGN KEY (capital) REFERENCES city(id)
);

CREATE TABLE IF NOT EXISTS country_language (
  country_code TEXT NOT NULL,
  language TEXT NOT NULL,
  is_official INTEGER NOT NULL DEFAULT 0,
  percentage REAL NOT NULL DEFAULT 0,
  FOREIGN KEY (country_code) REFERENCES country(code)
);

CREATE TABLE IF NOT EXISTS country_flag (
  code2 TEXT NOT NULL,
  emoji TEXT NOT NULL,
  unicode TEXT DEFAULT NULL
);

INSERT INTO city (id, name, country_code, district, population) VALUES
(129, 'Oranjestad', 'ABW', 'Aruba', 29034),
(1, 'Kabul', 'AFG', 'Kabol', 1780000),
(56, 'Luanda', 'AGO', 'Luanda', 2022000);

INSERT INTO country (code, name, continent, region, surface_area, indep_year, population, life_expectancy, gnp, gnp_old, local_name, government_form, head_of_state, capital, code2) VALUES
('ABW', 'Aruba', 'North America', 'Caribbean', 193.0, 1986, 103000, 78.4, 828.00, 793.00, 'Aruba', 'Nonmetropolitan Territory of The Netherlands', 'Willem-Alexander', 129, 'AW'),
('AFG', 'Afghanistan', 'Asia', 'Southern and Central Asia', 652090.0, 1919, 22720000, 45.9, 5976.00, NULL, 'Afganistan/Afqanestan', 'Islamic Emirate', 'Mohammad Omar', 1, 'AF'),
('AGO', 'Angola', 'Africa', 'Central Africa', 1246700.0, 1975, 12878000, 38.3, 6648.00, 7984.00, 'Angola', 'Republic', 'Jose Eduardo dos Santos', 56, 'AO');

INSERT INTO country_language (country_code, language, is_official, percentage) VALUES
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

INSERT INTO country_flag (code2, emoji, unicode) VALUES
('AW', '🇦🇼', 'U+1F1E6 U+1F1FC'),
('AF', '🇦🇫', 'U+1F1E6 U+1F1EB'),
('AO', '🇦🇴', 'U+1F1E6 U+1F1F4');
