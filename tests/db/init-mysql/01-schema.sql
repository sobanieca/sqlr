CREATE TABLE IF NOT EXISTS city (
  id INT NOT NULL AUTO_INCREMENT,
  name TEXT NOT NULL,
  country_code CHAR(3) NOT NULL,
  district TEXT NOT NULL,
  population INT NOT NULL DEFAULT 0,
  PRIMARY KEY (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS country (
  code CHAR(3) NOT NULL,
  name TEXT NOT NULL,
  continent ENUM('Asia','Europe','North America','Africa','Oceania','Antarctica','South America') NOT NULL DEFAULT 'Asia',
  region TEXT NOT NULL,
  surface_area FLOAT NOT NULL DEFAULT 0,
  indep_year SMALLINT DEFAULT NULL,
  population INT NOT NULL DEFAULT 0,
  life_expectancy FLOAT DEFAULT NULL,
  gnp DECIMAL(10,2) DEFAULT NULL,
  gnp_old DECIMAL(10,2) DEFAULT NULL,
  local_name TEXT NOT NULL,
  government_form TEXT NOT NULL,
  head_of_state TEXT DEFAULT NULL,
  capital INT DEFAULT NULL,
  code2 CHAR(2) NOT NULL,
  PRIMARY KEY (code),
  CONSTRAINT fk_country_capital FOREIGN KEY (capital) REFERENCES city(id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS country_language (
  country_code CHAR(3) NOT NULL,
  language TEXT NOT NULL,
  is_official BOOLEAN NOT NULL DEFAULT FALSE,
  percentage FLOAT NOT NULL DEFAULT 0,
  CONSTRAINT fk_country_language_code FOREIGN KEY (country_code) REFERENCES country(code)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS country_flag (
  code2 CHAR(2) NOT NULL,
  emoji TEXT NOT NULL,
  unicode TEXT DEFAULT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- Insert cities
INSERT INTO city (id, name, country_code, district, population) VALUES
(129, 'Oranjestad', 'ABW', 'Aruba', 29034),
(1, 'Kabul', 'AFG', 'Kabol', 1780000),
(56, 'Luanda', 'AGO', 'Luanda', 2022000);

-- Insert countries
INSERT INTO country (code, name, continent, region, surface_area, indep_year, population, life_expectancy, gnp, gnp_old, local_name, government_form, head_of_state, capital, code2) VALUES
('ABW', 'Aruba', 'North America', 'Caribbean', 193, 1986, 103000, 78.4, 828.00, 793.00, 'Aruba', 'Nonmetropolitan Territory of The Netherlands', 'Willem-Alexander', 129, 'AW'),
('AFG', 'Afghanistan', 'Asia', 'Southern and Central Asia', 652090, 1919, 22720000, 45.9, 5976.00, NULL, 'Afganistan/Afqanestan', 'Islamic Emirate', 'Mohammad Omar', 1, 'AF'),
('AGO', 'Angola', 'Africa', 'Central Africa', 1246700, 1975, 12878000, 38.3, 6648.00, 7984.00, 'Angola', 'Republic', 'Jose Eduardo dos Santos', 56, 'AO');

-- Insert country languages
INSERT INTO country_language (country_code, language, is_official, percentage) VALUES
('ABW', 'Dutch', TRUE, 5.3),
('ABW', 'English', FALSE, 9.5),
('ABW', 'Papiamento', FALSE, 76.7),
('ABW', 'Spanish', FALSE, 7.4),
('AFG', 'Balochi', FALSE, 0.9),
('AFG', 'Dari', TRUE, 32.1),
('AFG', 'Pashto', TRUE, 52.4),
('AFG', 'Turkmenian', FALSE, 1.9),
('AFG', 'Uzbek', FALSE, 8.9),
('AGO', 'Kimbundu', FALSE, 25.0),
('AGO', 'Mbundu', FALSE, 22.0),
('AGO', 'Ovimbundu', FALSE, 37.0);

-- Insert country flags
INSERT INTO country_flag (code2, emoji, unicode) VALUES
('AW', '🇦🇼', 'U+1F1E6 U+1F1FC'),
('AF', '🇦🇫', 'U+1F1E6 U+1F1EB'),
('AO', '🇦🇴', 'U+1F1E6 U+1F1F4');
