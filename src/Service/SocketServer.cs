using System;
using System.IO;
using System.Net.Sockets;
using System.Threading.Tasks;
using Gotham.Models;
using Gotham.Utils;

namespace Gotham
{
	public class SocketServer
	{
		public static bool Running = true;

		private static FileStream? lockFile;
		private static Socket? listeningSocket;

		public static void Listen(string unixSocket)
		{
			try
			{
				lockFile = new FileStream(unixSocket + ".lock", FileMode.OpenOrCreate, FileAccess.Write, FileShare.None);
			}
			catch (IOException ex) when (ex.HResult == 11)
			{
				// Lock file is being used by another process.
				// Another instance of gotham is running
				Console.WriteLine($"Another instance of {Constants.AppName} is running on the same socket. Please close that before running it again");
				return;
			}

			// File lock is aquired. If the unix socket exists, then it's clearly a dangling socket. Feel free to delete it
			if(File.Exists(unixSocket))
				File.Delete(unixSocket);

			listeningSocket = new Socket(AddressFamily.Unix, SocketType.Stream, ProtocolType.Unspecified);
			var socketEndPoint = new UnixDomainSocketEndPoint(unixSocket);

			try
			{
				listeningSocket.Bind(socketEndPoint);
				listeningSocket.Listen(int.MaxValue);

				while (Running)
				{
					var client = listeningSocket.Accept();
					Task.Run(() => PollForModuleData(client));
				}
			}
			catch (Exception e)
			{
				Console.WriteLine(e.Message);
				Console.WriteLine(e.StackTrace);
				listeningSocket.Close();
			}
		}

		public static void Stop()
		{
			Running = false;
			listeningSocket?.Dispose();
			lockFile?.Dispose();
		}

		private static void PollForModuleData(Socket client)
		{
			try
			{
				var module = new Module(client);
				Console.WriteLine("Polling for data");
				module.PollForData();
			}
			catch (Exception e)
			{
				Console.WriteLine(e.ToString());
			}
		}
	}
}