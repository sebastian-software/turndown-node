export interface Options {
  headingStyle?: "setext" | "atx";
  hr?: string;
  bulletListMarker?: "*" | "-" | "+";
  codeBlockStyle?: "indented" | "fenced";
  fence?: "```" | "~~~";
  emDelimiter?: "_" | "*";
  strongDelimiter?: "**" | "__";
  linkStyle?: "inlined" | "referenced";
  linkReferenceStyle?: "full" | "collapsed" | "shortcut";
}

export interface Rule {
  filter: string | string[] | ((node: Node) => boolean);
  replacement: (content: string, node: Node, options: Options) => string;
}

export class TurndownService {
  constructor(options?: Options);
  turndown(html: string): string;
  addRule(key: string, rule: Rule): this;
  keep(filter: string | string[]): this;
  remove(filter: string | string[]): this;
  use(plugin: (service: TurndownService) => void): this;
  escape(str: string): string;
}

export default TurndownService;
