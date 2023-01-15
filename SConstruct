AddOption(
	"--env",
	type="string",
	nargs=1
)

envtype = GetOption("env")

flags = ["-Wall",  "-Wextra"]

if envtype != "dev":
	flags.append("-O3")
else:
	flags.append("-g3")
	flags.append("-O0")

env = Environment(CCFLAGS=" ".join(flags), CC="clang")
env.VariantDir('build', 'src', duplicate=0)
env.Program('build/ena', Glob('build/*.c'))