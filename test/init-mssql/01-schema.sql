CREATE DATABASE [world-db];
GO

USE [world-db];
GO

CREATE TABLE city (
  id INT NOT NULL IDENTITY(1,1),
  name NVARCHAR(255) NOT NULL,
  country_code CHAR(3) NOT NULL,
  district NVARCHAR(255) NOT NULL,
  population INT NOT NULL DEFAULT 0,
  PRIMARY KEY (id)
);
GO

CREATE TABLE country (
  code CHAR(3) NOT NULL,
  name NVARCHAR(255) NOT NULL,
  continent NVARCHAR(50) NOT NULL DEFAULT 'Asia',
  region NVARCHAR(255) NOT NULL,
  surface_area FLOAT NOT NULL DEFAULT 0,
  indep_year SMALLINT NULL,
  population INT NOT NULL DEFAULT 0,
  life_expectancy FLOAT NULL,
  gnp DECIMAL(10,2) NULL,
  gnp_old DECIMAL(10,2) NULL,
  local_name NVARCHAR(255) NOT NULL,
  government_form NVARCHAR(255) NOT NULL,
  head_of_state NVARCHAR(255) NULL,
  capital INT NULL,
  code2 CHAR(2) NOT NULL,
  PRIMARY KEY (code),
  CONSTRAINT fk_country_capital FOREIGN KEY (capital) REFERENCES city(id)
);
GO

CREATE TABLE country_language (
  country_code CHAR(3) NOT NULL,
  language NVARCHAR(255) NOT NULL,
  is_official BIT NOT NULL DEFAULT 0,
  percentage FLOAT NOT NULL DEFAULT 0,
  CONSTRAINT fk_country_language_code FOREIGN KEY (country_code) REFERENCES country(code)
);
GO

CREATE TABLE country_flag (
  code2 CHAR(2) NOT NULL,
  emoji NVARCHAR(50) NOT NULL,
  unicode NVARCHAR(255) NULL
);
GO

-- Insert cities
SET IDENTITY_INSERT city ON;
INSERT INTO city (id, name, country_code, district, population) VALUES
(129, 'Oranjestad', 'ABW', 'Aruba', 29034),
(1, 'Kabul', 'AFG', 'Kabol', 1780000),
(56, 'Luanda', 'AGO', 'Luanda', 2022000);
SET IDENTITY_INSERT city OFF;
GO

-- Insert countries
INSERT INTO country (code, name, continent, region, surface_area, indep_year, population, life_expectancy, gnp, gnp_old, local_name, government_form, head_of_state, capital, code2) VALUES
('ABW', 'Aruba', 'North America', 'Caribbean', 193, 1986, 103000, 78.4, 828.00, 793.00, 'Aruba', 'Nonmetropolitan Territory of The Netherlands', 'Willem-Alexander', 129, 'AW'),
('AFG', 'Afghanistan', 'Asia', 'Southern and Central Asia', 652090, 1919, 22720000, 45.9, 5976.00, NULL, 'Afganistan/Afqanestan', 'Islamic Emirate', 'Mohammad Omar', 1, 'AF'),
('AGO', 'Angola', 'Africa', 'Central Africa', 1246700, 1975, 12878000, 38.3, 6648.00, 7984.00, 'Angola', 'Republic', 'Jose Eduardo dos Santos', 56, 'AO');
GO

-- Insert country languages
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
GO

-- Insert country flags
INSERT INTO country_flag (code2, emoji, unicode) VALUES
('AW', N'🇦🇼', 'U+1F1E6 U+1F1FC'),
('AF', N'🇦🇫', 'U+1F1E6 U+1F1EB'),
('AO', N'🇦🇴', 'U+1F1E6 U+1F1F4');
GO
