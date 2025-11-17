import { describe, it, expect } from 'vitest';
import type { SearchQuery } from '../index';

/**
 * Tests for Regex Search feature TypeScript types.
 * These tests verify that the SearchQuery interface properly supports the regexSearch field.
 */

describe('SearchQuery TypeScript Types - Regex Search', () => {
  describe('SearchQuery Interface', () => {
    it('should accept regexSearch as optional boolean', () => {
      const query: SearchQuery = {
        query: 'test',
        regexSearch: true,
      };

      expect(query.query).toBe('test');
      expect(query.regexSearch).toBe(true);
    });

    it('should accept regexSearch as false', () => {
      const query: SearchQuery = {
        query: 'test',
        regexSearch: false,
      };

      expect(query.regexSearch).toBe(false);
    });

    it('should allow regexSearch to be undefined (optional)', () => {
      const query: SearchQuery = {
        query: 'test',
      };

      expect(query.regexSearch).toBeUndefined();
    });

    it('should work with all other SearchQuery fields', () => {
      const query: SearchQuery = {
        query: 'Crawler',
        project: 'klask',
        version: '1.0',
        extension: 'rs',
        maxResults: 50,
        offset: 0,
        fuzzySearch: false,
        regexSearch: true,
      };

      expect(query.query).toBe('Crawler');
      expect(query.project).toBe('klask');
      expect(query.version).toBe('1.0');
      expect(query.extension).toBe('rs');
      expect(query.maxResults).toBe(50);
      expect(query.offset).toBe(0);
      expect(query.fuzzySearch).toBe(false);
      expect(query.regexSearch).toBe(true);
    });

    it('should allow creating SearchQuery with both fuzzySearch and regexSearch', () => {
      const query: SearchQuery = {
        query: 'test',
        fuzzySearch: true,
        regexSearch: false,
      };

      expect(query.fuzzySearch).toBe(true);
      expect(query.regexSearch).toBe(false);
    });

    it('should allow creating SearchQuery with minimal required fields', () => {
      const query: SearchQuery = {
        query: 'test',
      };

      expect(query.query).toBe('test');
      expect(query.regexSearch).toBeUndefined();
      expect(query.fuzzySearch).toBeUndefined();
    });

    it('should not require regexSearch field', () => {
      // This should compile without errors
      const query: SearchQuery = {
        query: 'test',
        project: 'project',
      };

      expect(query.query).toBe('test');
      expect(query.project).toBe('project');
      // regexSearch is not required
      expect(query.regexSearch).toBeUndefined();
    });

    it('should support regex pattern strings in query field', () => {
      const query: SearchQuery = {
        query: '^Crawler.*$',
        regexSearch: true,
      };

      expect(query.query).toBe('^Crawler.*$');
      expect(query.regexSearch).toBe(true);
    });
  });

  describe('Type Compatibility', () => {
    it('should be assignable to Record<string, any>', () => {
      const query: SearchQuery = {
        query: 'test',
        regexSearch: true,
      };

      const record: Record<string, any> = query;

      expect(record.query).toBe('test');
      expect(record.regexSearch).toBe(true);
    });

    it('should support spreading into a new object', () => {
      const baseQuery: SearchQuery = {
        query: 'test',
      };

      const extendedQuery = {
        ...baseQuery,
        regexSearch: true,
        maxResults: 20,
      };

      expect(extendedQuery.query).toBe('test');
      expect(extendedQuery.regexSearch).toBe(true);
      expect(extendedQuery.maxResults).toBe(20);
    });

    it('should support partial updates', () => {
      let query: SearchQuery = {
        query: 'test',
        regexSearch: false,
      };

      // Update to enable regex
      query = {
        ...query,
        regexSearch: true,
      };

      expect(query.regexSearch).toBe(true);
    });

    it('should work in conditional type checking', () => {
      const query: SearchQuery = {
        query: 'test',
        regexSearch: true,
      };

      if (query.regexSearch) {
        expect(query.regexSearch).toBe(true);
      } else {
        expect(query.regexSearch).toBeFalsy();
      }
    });
  });

  describe('Optional Fields Handling', () => {
    it('should handle undefined regexSearch gracefully', () => {
      const query: SearchQuery = {
        query: 'test',
        regexSearch: undefined,
      };

      expect(query.regexSearch).toBeUndefined();
    });

    it('should allow omitting optional fields', () => {
      const query: SearchQuery = {
        query: 'test',
        // All other fields omitted
      };

      const hasRegex = query.regexSearch === undefined;
      expect(hasRegex).toBe(true);
    });

    it('should correctly type optional boolean as boolean | undefined', () => {
      const query: SearchQuery = {
        query: 'test',
      };

      const regexValue: boolean | undefined = query.regexSearch;
      expect(regexValue).toBeUndefined();
    });

    it('should support optional chaining on regexSearch', () => {
      const query: SearchQuery = {
        query: 'test',
      };

      const value = query.regexSearch ?? false;
      expect(value).toBe(false);
    });
  });

  describe('Type Guards', () => {
    it('should allow checking if regexSearch is true', () => {
      const queries: SearchQuery[] = [
        { query: 'test1', regexSearch: true },
        { query: 'test2', regexSearch: false },
        { query: 'test3' },
      ];

      const regexQueries = queries.filter(q => q.regexSearch === true);
      expect(regexQueries).toHaveLength(1);
      expect(regexQueries[0].query).toBe('test1');
    });

    it('should allow checking if regexSearch is explicitly false', () => {
      const queries: SearchQuery[] = [
        { query: 'test1', regexSearch: true },
        { query: 'test2', regexSearch: false },
        { query: 'test3' },
      ];

      const nonRegexQueries = queries.filter(q => q.regexSearch === false);
      expect(nonRegexQueries).toHaveLength(1);
      expect(nonRegexQueries[0].query).toBe('test2');
    });

    it('should allow filtering queries without regexSearch field', () => {
      const queries: SearchQuery[] = [
        { query: 'test1', regexSearch: true },
        { query: 'test2', regexSearch: false },
        { query: 'test3' },
      ];

      const queriesWithoutRegex = queries.filter(q => q.regexSearch === undefined);
      expect(queriesWithoutRegex).toHaveLength(1);
      expect(queriesWithoutRegex[0].query).toBe('test3');
    });
  });

  describe('Backward Compatibility', () => {
    it('should accept existing code that does not use regexSearch', () => {
      // Old code that doesn't know about regexSearch
      const query: SearchQuery = {
        query: 'test',
        project: 'my-project',
        version: '1.0',
        extension: 'ts',
        maxResults: 50,
      };

      expect(query.query).toBe('test');
      expect(query.project).toBe('my-project');
      // regexSearch is not set, should be undefined
      expect(query.regexSearch).toBeUndefined();
    });

    it('should not break existing SearchQuery usage', () => {
      const queries: SearchQuery[] = [
        { query: 'function' },
        { query: 'class', project: 'myapp' },
        { query: 'import', version: '2.0' },
      ];

      expect(queries).toHaveLength(3);
      expect(queries.every(q => q.query)).toBe(true);
    });

    it('should allow gradual adoption of regexSearch', () => {
      // Some queries use regex, some don't
      const regularQuery: SearchQuery = { query: 'test' };
      const regexQuery: SearchQuery = { query: '^test.*', regexSearch: true };

      expect(regularQuery.regexSearch).toBeUndefined();
      expect(regexQuery.regexSearch).toBe(true);
    });
  });

  describe('Interface Satisfaction', () => {
    it('should satisfy SearchQuery when all fields are provided', () => {
      const query: SearchQuery = {
        query: 'Crawler',
        project: 'klask',
        version: '1.0',
        extension: 'rs',
        maxResults: 20,
        offset: 0,
        fuzzySearch: false,
        regexSearch: true,
      };

      // Type checking - this should compile
      expect(query).toBeDefined();
      expect(query.regexSearch).toBe(true);
    });

    it('should satisfy SearchQuery with minimal required fields', () => {
      const query: SearchQuery = {
        query: 'test',
      };

      expect(query).toBeDefined();
      expect(query.query).toBe('test');
    });
  });

  describe('Type Inference', () => {
    it('should infer regexSearch type as optional boolean', () => {
      const query = {
        query: 'test',
        regexSearch: true,
      } as const;

      // Type should be inferred correctly
      expect(typeof query.regexSearch).toBe('boolean');
    });

    it('should allow type assertion to SearchQuery', () => {
      const obj = { query: 'test', regexSearch: true };
      const query = obj as SearchQuery;

      expect(query.regexSearch).toBe(true);
    });
  });

  describe('Documentation Examples', () => {
    it('should support basic regex search example', () => {
      // Example: user wants to search for functions starting with "handle"
      const query: SearchQuery = {
        query: '^handle.*',
        regexSearch: true,
      };

      expect(query.query).toBe('^handle.*');
      expect(query.regexSearch).toBe(true);
    });

    it('should support regex + fuzzy combination', () => {
      // Example: user wants both regex and fuzzy matching
      const query: SearchQuery = {
        query: 'serv',
        fuzzySearch: true,
        regexSearch: false, // not enabled simultaneously in this case
      };

      expect(query.fuzzySearch).toBe(true);
      expect(query.regexSearch).toBe(false);
    });

    it('should support regex with filters', () => {
      // Example: user wants regex search in specific projects/versions
      const query: SearchQuery = {
        query: 'fn [a-z]+\\(.*\\)',
        regexSearch: true,
        project: 'klask-rs',
        version: 'main',
        extension: 'rs',
      };

      expect(query.query).toBeTruthy();
      expect(query.regexSearch).toBe(true);
      expect(query.project).toBe('klask-rs');
    });
  });
});
