using System;
using System.IO;

namespace Gotham
{
	public class Program
	{
		public static void Main(string[] args)
		{
			OnStart(args);
		}

		private static void OnStart(string[] args)
		{
			Console.WriteLine("Creating the socket");
			Console.CancelKeyPress += OnExit;
			SocketServer.Listen();
		}

		private static void OnExit(object sender, ConsoleCancelEventArgs e)
		{
			SocketServer.Running = false;
			if (File.Exists(SocketServer.UnixSocket))
				File.Delete(SocketServer.UnixSocket);
		}
	}
}
