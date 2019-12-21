using System;
using System.IO;
using CommandLine;
using Gotham.Utils;

namespace Gotham
{
	public class Program
	{
		public static string UnixSocket { get; private set; } = Constants.DefaultUnixSocket;
		public static string ModulesDirectory { get; private set; } = Constants.DefaultModulesDirectory;

		public static void Main(string[] args)
		{
			new Parser((settings) =>
			{
				settings.AutoVersion = false;
				settings.IgnoreUnknownArguments = false;
			})
				.ParseArguments<CliOptions>(args)
				.WithParsed<CliOptions>(OnStart);
		}

		private static void OnStart(CliOptions options)
		{
			// Parse any and all command line arguments here
			if (options.Version)
			{
				Console.WriteLine(Constants.AppVersion);
				return;
			}
			UnixSocket = options.SocketLocation;
			ModulesDirectory = options.ModulesLocation;

			Console.CancelKeyPress += OnExit;
			SocketServer.Listen(UnixSocket);
		}

		private static void OnExit(object sender, ConsoleCancelEventArgs e)
		{
			try
			{
				Console.WriteLine("Exiting");
				SocketServer.Stop();
				if (File.Exists(UnixSocket + ".lock"))
					File.Delete(UnixSocket + ".lock");
				if (File.Exists(UnixSocket))
					File.Delete(UnixSocket);
			}
			catch (Exception ex)
			{
				Console.WriteLine(ex);
			}
		}
	}

	public class CliOptions
	{
		[Option('s', "socket-location", Required = false, HelpText = "Sets the location of the socket to be created", MetaValue = "FILE", Default = Constants.DefaultUnixSocket)]
		public string SocketLocation { get; set; } = Constants.DefaultUnixSocket;

		[Option('V', Required = false, HelpText = "Sets the level of verbosity", Default = 0)]
		public int Verbosity { get; set; } = 0;

		[Option('m', "modules-location", Required = false, HelpText = "Sets the location of the modules to run", MetaValue = "DIR", Default = Constants.DefaultModulesDirectory)]
		public string ModulesLocation { get; set; } = Constants.DefaultModulesDirectory;

		[Option('v', "version", Required = false, HelpText = "Prints version information")]
		public bool Version { get; set; } = false;
	}
}
