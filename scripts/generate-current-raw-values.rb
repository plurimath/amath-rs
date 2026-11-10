require "json"
require "digest"
require "amath"
SOURCE_ROOT = File.expand_path(ARGV.fetch(0))
INPUT_PATH = File.expand_path(ARGV.fetch(1))
OUTPUT_PATH = File.expand_path(ARGV.fetch(2))
def plain(value)
  case value
  when Hash
    value.each_with_object({}) do |(key, child), result|
      name = key.to_s
      raise "duplicate normalized key #{name}" if result.key?(name)
      result[name] = plain(child)
    end
  when Array then value.map { |child| plain(child) }
  when String, Numeric, NilClass, TrueClass, FalseClass then value
  when Symbol then value.to_s
  else
    raise "unsupported raw leaf #{value.class}" unless value.class.name.match?(/\A(?:Parsanol|Parslet)::Slice\z/)
    value.to_s
  end
end
expected_head = "a4ed231057aeb98022f5c825807e888946a9e71c"
actual_head = IO.popen(["git", "-C", SOURCE_ROOT, "rev-parse", "HEAD"], &:read).strip
raise "source checkout must match the pinned producer" unless actual_head == expected_head
raise "source checkout must be clean" unless IO.popen(["git", "-C", SOURCE_ROOT, "status", "--porcelain"], &:read).empty?
original = JSON.parse(File.read(INPUT_PATH))
raise "expected all160 archived inputs" unless original.fetch("cases").length == 160
rows = original.fetch("cases").map do |row|
  input, decimal = row.fetch("input"), row.fetch("decimal", ".")
  results = [:native, :ruby].to_h do |engine|
    [engine, plain(Amath::Parser.raw(input, decimal: decimal, engine: engine))]
  end
  raise "engine disagreement #{row.fetch('id')}" unless JSON.generate(results[:native]) == JSON.generate(results[:ruby])
  {"id" => row.fetch("id"), "input" => input, "decimal" => decimal, "tree" => results[:native]}
end
files = %w[lib/amath/parser.rb lib/amath/raw_tree.rb lib/amath/renester.rb lib/amath/engine.rb grammars/asciimath.parg artifacts/asciimath.json Gemfile.lock]
head = IO.popen(["git", "-C", SOURCE_ROOT, "rev-parse", "HEAD"], &:read).strip
metadata = {
 "schema" => "amath-current-raw-values/v1", "profile" => "Amath::Parser.raw@#{head}",
 "producer" => {"repository" => "https://github.com/plurimath/amath", "commit" => head,
 "entry_point" => "Amath::Parser.raw", "engines" => ["native", "ruby"], "engine_agreement" => true,
 "ruby" => RUBY_VERSION, "parsanol" => Gem.loaded_specs.fetch("parsanol").version.to_s,
 "source_files" => files.to_h { |name| [name, Digest::SHA256.file(File.join(SOURCE_ROOT,name)).hexdigest] },
 "generator_sha256" => Digest::SHA256.file(__FILE__).hexdigest,
 "native_modules" => $LOADED_FEATURES.grep(/parsanol.*\.(so|bundle)\z/).to_h { |path| [path.delete_prefix(Gem.loaded_specs.fetch("parsanol").full_gem_path + "/"), Digest::SHA256.file(path).hexdigest] }},
 "archived_reference" => {"repository" => original.fetch("repository"), "commit" => original.fetch("commit"),
 "generation" => original.fetch("generation"), "boundary" => original.fetch("boundary"), "sha256" => Digest::SHA256.file(INPUT_PATH).hexdigest},
 "cases" => rows, "excluded" => original.fetch("excluded")
}
File.write(OUTPUT_PATH, JSON.pretty_generate(metadata) + "\n")
puts JSON.generate(cases: rows.length, excluded: metadata.fetch("excluded").length, engines_agree: true, source_commit: head, parsanol: metadata.dig("producer","parsanol"), output_sha256: Digest::SHA256.file(OUTPUT_PATH).hexdigest)
