using System;
using System.Collections.Generic;
using System.Net.Sockets;
using System.Text;
using System.Threading;
using Gotham.Service;
using Gotham.Utils;

namespace Gotham.Models
{
	public class Module
	{
		public static Module GothamModule = new Module(new Socket(AddressFamily.Unix, SocketType.Stream, ProtocolType.Unspecified))
		{
			Registered = true,
			ModuleID = Constants.AppName,
			Version = Constants.Version,
		};

		static Module()
		{
			DataHandler.RegisteredModules.Add(GothamModule.ModuleID, GothamModule);
		}

		public bool Registered { get; internal set; } = false;
		public string ModuleID { get; internal set; } = "undefined";
		public string Version { get; internal set; } = "0.0.0";
		public Dictionary<string, string> Dependencies { get; internal set; } = new Dictionary<string, string>();
		public List<string> DeclaredFunctions { get; internal set; } = new List<string>();
		public List<string> RegisteredHooks { get; internal set; } = new List<string>();

		internal string RegistrationRequestID { get; set; } = string.Empty;

		private Socket clientSocket;
		private StringBuilder buffer = new StringBuilder();

		public Module(Socket client)
		{
			clientSocket = client;
		}

		public void PollForData()
		{
			while (clientSocket.Connected)
			{
				var dataAvailableToRead = clientSocket.Available;

				if (dataAvailableToRead == 0)
				{
					Thread.Sleep(5);
					continue;
				}

				// Fill read buffer with data coming from the socket
				var readBuffer = new byte[dataAvailableToRead];
				clientSocket.Receive(readBuffer, dataAvailableToRead, SocketFlags.None);

				buffer.Append(Encoding.UTF8.GetString(readBuffer));
				var input = buffer.ToString();
				if (!input.Contains('\n'))
					continue;

				// Split the incoming data by \n to separate requests
				var jsons = input.Split('\n', StringSplitOptions.RemoveEmptyEntries);

				// If the last request didn't end with a \n, then it's probably an incomplete one
				// So, don't process the last request (iterate to length - 1, allowing it to fill the buffer)
				var didRecieveCompleteRequest = input.EndsWith('\n');
				var requestCount = didRecieveCompleteRequest ? jsons.Length : jsons.Length - 1;

				for (var i = 0; i < requestCount; i++)
				{
					var request = jsons[i];
					DataHandler.HandleRequest(this, request);
				}
				buffer.Clear();

				// if you didn't recieve a complete request, keep the last data
				// to allow the new data to append to it
				if (!didRecieveCompleteRequest)
					buffer.Append(jsons[jsons.Length - 1]);
			}
			OnDisconnected();
		}

		internal void SendBytes(byte[] bytesToSend)
		{
			clientSocket.Send(bytesToSend, 0, bytesToSend.Length, SocketFlags.None);
		}

		private void OnDisconnected()
		{
			DataHandler.OnModuleDisconnected(this);
		}
	}
}