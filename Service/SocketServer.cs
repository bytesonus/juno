using System;
using System.Net.Sockets;
using System.Threading.Tasks;
using Gotham.Models;

namespace Gotham
{
	public class SocketServer
	{
		public const string UnixSocket = "../gotham.sock";
		public static bool Running = true;

		public static void Listen()
		{
			var listeningSocket = new Socket(AddressFamily.Unix, SocketType.Stream, ProtocolType.Unspecified);
			var socketEndPoint = new UnixDomainSocketEndPoint(UnixSocket);

			try
			{
				listeningSocket.Bind(socketEndPoint);
				listeningSocket.Listen(int.MaxValue);

				while (Running)
				{
					var client = listeningSocket.Accept();
					Console.WriteLine("Connection accepted");
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