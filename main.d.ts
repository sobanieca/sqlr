/**
 * SQLR - A command-line tool for interacting with SQL databases.
 *
 * This module provides:
 * - SQL query execution against PostgreSQL databases
 * - Database schema description
 * - Connection management with optional encryption
 *
 * @example
 * ```bash
 * # Show version
 * sqlr --version
 *
 * # Show help
 * sqlr --help
 *
 * # Run a query
 * sqlr query -t postgresql -s "postgres://..." -q "SELECT 1"
 * ```
 */

/**
 * No exports - this module executes commands when imported/executed.
 */
export {};
