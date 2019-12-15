using System.Collections.Generic;
using Newtonsoft.Json.Linq;

namespace Gotham.Models
{
	public class ModuleRequestHandler
	{
		public RequestHandlerDelegate Handler;
		public List<ModuleRequestParameter> ParametersRequired { get; } = new List<ModuleRequestParameter>();

		public ModuleRequestHandler(RequestHandlerDelegate handler)
		{
			Handler = handler;
		}
	}

	public class ModuleRequestParameter
	{
		public string ParameterName { get; }
		public JTokenType ParameterType { get; set; } = JTokenType.String;
		public bool Optional { get; set; } = false;

		public ModuleRequestParameter(string ParameterName)
		{
			this.ParameterName = ParameterName;
		}
	}

	public delegate void RequestHandlerDelegate(Module module, string requestId, JObject request);
}